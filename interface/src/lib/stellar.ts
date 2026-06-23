import { rpc, TransactionBuilder, Networks, Contract, xdr, Address, nativeToScVal, scValToNative } from "@stellar/stellar-sdk";
import { signTransaction } from "@stellar/freighter-api";

const RPC_URL = "https://soroban-testnet.stellar.org";
const NETWORK_PASSPHRASE = Networks.TESTNET;

export const server = new rpc.Server(RPC_URL, { allowHttp: true });

export const MAIN_CONTRACT_ID = process.env.NEXT_PUBLIC_MAIN_ID || "CD6NZ7RHK2IUCP4CM3WHHNK22BBXPYIZUX6ISHZAGXXKTI6ZMN6Q5FFR";
export const USDC_CONTRACT_ID = process.env.NEXT_PUBLIC_MOCK_USDC_ID || "CAEIJB5RB4MQ2NBT6QVGZETKR6CBWPZBFKJX5472ACPFLDYJNHEFK6KB";

/**
 * Human-readable messages for the Soroban contract error codes
 * (kept in sync with `soroban/src/types.rs` -> enum Error).
 */
const CONTRACT_ERRORS: Record<number, string> = {
  1: "Contract already initialized",
  2: "Contract not initialized",
  3: "Unauthorized",
  4: "Invalid amount",
  5: "Insufficient balance",
  6: "The lending pool has insufficient liquidity to fund this loan",
  7: "Loan not found",
  8: "Loan is not active",
  9: "Loan has expired",
  10: "Invalid asset",
  11: "Invalid skin-in-the-game bond",
  12: "ZK proof verification failed",
  13: "Repayment amount is too small",
  14: "Loan has already been repaid",
  15: "You are not the borrower of this loan",
  16: "Token transfer failed (check your balance and trustline)",
  17: "Arithmetic overflow",
  18: "Loan is not yet due",
  19: "This invoice has already been used (nullifier spent)",
};

/**
 * Turns a raw Soroban host error string like
 * `HostError: Error(Contract, #6) ...` into a friendly message.
 */
export function parseContractError(raw: unknown): string {
  const text = String(raw ?? "");
  const match = text.match(/Error\(Contract,\s*#(\d+)\)/);
  if (match) {
    const code = parseInt(match[1], 10);
    return CONTRACT_ERRORS[code] || `Contract error #${code}`;
  }
  return text;
}

/**
 * Helper to build and submit a Soroban transaction via Freighter.
 */
export async function submitContractCall(
  publicKey: string,
  contractId: string,
  method: string,
  args: xdr.ScVal[] = []
) {
  try {
    const account = await server.getAccount(publicKey);
    const contract = new Contract(contractId);

    const tx = new TransactionBuilder(account, {
      fee: "10000",
      networkPassphrase: NETWORK_PASSPHRASE,
    })
      .addOperation(contract.call(method, ...args))
      .setTimeout(30)
      .build();

    // Simulate to get footprint & updated fee
    const simulated = await server.simulateTransaction(tx);
    
    if (rpc.Api.isSimulationError(simulated)) {
      throw new Error(parseContractError(simulated.error));
    }

    const assembledTx = rpc.assembleTransaction(tx, simulated).build();
    
    // Sign with Freighter
    const signResponse = await signTransaction(assembledTx.toXDR(), {
      networkPassphrase: NETWORK_PASSPHRASE
    });

    if (signResponse.error) {
      throw new Error(signResponse.error);
    }

    const txToSubmit = TransactionBuilder.fromXDR(signResponse.signedTxXdr, NETWORK_PASSPHRASE);
    const sendResponse = await server.sendTransaction(txToSubmit as any);
    
    if (sendResponse.status === "ERROR") {
      throw new Error(`Transaction failed: ${JSON.stringify(sendResponse)}`);
    }

    // Poll for status
    let statusResponse = await server.getTransaction(sendResponse.hash);
    while (statusResponse.status === "NOT_FOUND") {
      await new Promise(resolve => setTimeout(resolve, 2000));
      statusResponse = await server.getTransaction(sendResponse.hash);
    }
    
    if (statusResponse.status === "FAILED") {
      throw new Error("Transaction failed on chain");
    }

    // Decode the contract's return value (e.g. the loan_id from request_loan)
    let returnValue: any = null;
    const retval = (statusResponse as any).returnValue;
    if (retval) {
      try {
        returnValue = scValToNative(retval);
      } catch {
        returnValue = null;
      }
    }

    return { hash: sendResponse.hash, returnValue };
  } catch (error) {
    console.error("submitContractCall error:", error);
    throw error;
  }
}

/**
 * Helper to simulate a read-only contract call
 */
export async function readContract(
  contractId: string,
  method: string,
  args: xdr.ScVal[] = []
) {
  try {
    const contract = new Contract(contractId);
    
    // For read-only calls we need a source account, but for simulation any address works.
    // Using a dummy zero address for simulation since it's just reading data.
    const dummyAccount = "GA7YDROPWQ25U4F7TKYZIBW22VDE3KUKD22CYYBHRNUDVBMY3E6Z2KXY"; 
    
    const account = await server.getAccount(dummyAccount).catch(() => null);
    
    // If dummy account doesn't exist on testnet, we'll construct a mock account
    let tx;
    if (account) {
      tx = new TransactionBuilder(account, {
        fee: "100",
        networkPassphrase: NETWORK_PASSPHRASE,
      })
      .addOperation(contract.call(method, ...args))
      .setTimeout(30)
      .build();
    } else {
      // Create a dummy account builder
      const dummyObj = {
        accountId: () => dummyAccount,
        sequenceNumber: () => "0",
        incrementSequenceNumber: () => {}
      } as any;
      
      tx = new TransactionBuilder(dummyObj, {
        fee: "100",
        networkPassphrase: NETWORK_PASSPHRASE,
      })
      .addOperation(contract.call(method, ...args))
      .setTimeout(30)
      .build();
    }

    const simulated = await server.simulateTransaction(tx);
    if (rpc.Api.isSimulationError(simulated)) {
      console.warn("Read simulation error:", simulated.error);
      return null;
    }
    
    if (rpc.Api.isSimulationSuccess(simulated)) {
  if (!simulated.result) {
    return null;
  }
  return scValToNative(simulated.result.retval);
    }
    
    return null;
  } catch (error) {
    console.error("readContract error:", error);
    return null;
  }
}
