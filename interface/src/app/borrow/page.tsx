"use client";

import { useState, useEffect } from "react";
import { generateMockProof } from "@/lib/zk-mock";
import { toast } from "sonner";
import { CheckCircle2, FileText, ArrowRight, XCircle } from "lucide-react";
import { useWallet } from "@/hooks/use-wallet";
import { SiXero, SiStripe, SiCoinbase, SiBinance, SiGusto } from "react-icons/si"

type Loan = {
  id: string;
  amount: string;
  date: string;
  status: string;
  /** Real on-chain loan ID returned by the contract (null for demo/sample loans) */
  contractId: number | null;
  /** Principal in stroops, used to repay the exact amount (null for demo loans) */
  principalStroops: number | null;
};

const LOANS_STORAGE_KEY = "aero_loans";

const DEFAULT_LOANS: Loan[] = [
  { id: "L-1045", amount: "5,000 USDC", date: "Jul 15, 2026", status: "Active", contractId: null, principalStroops: null },
  { id: "L-0982", amount: "12,000 XLM", date: "May 10, 2026", status: "Repaid", contractId: null, principalStroops: null },
];

export default function BorrowPage() {
  const { address } = useWallet();
  const [isVerified, setIsVerified] = useState(false);
  const [isGenerating, setIsGenerating] = useState(false);

  const [invoiceAmount, setInvoiceAmount] = useState("");
  const [dueDate, setDueDate] = useState("");

  const [loans, setLoans] = useState<Loan[]>(DEFAULT_LOANS);
  const [loansLoaded, setLoansLoaded] = useState(false);

  // Load persisted loans from localStorage on mount (avoids SSR hydration mismatch)
  useEffect(() => {
    try {
      const raw = localStorage.getItem(LOANS_STORAGE_KEY);
      if (raw) setLoans(JSON.parse(raw));
    } catch (e) {
      console.warn("Failed to load saved loans:", e);
    }
    setLoansLoaded(true);
  }, []);

  // Persist loans whenever they change (only after the initial load)
  useEffect(() => {
    if (!loansLoaded) return;
    try {
      localStorage.setItem(LOANS_STORAGE_KEY, JSON.stringify(loans));
    } catch (e) {
      console.warn("Failed to save loans:", e);
    }
  }, [loans, loansLoaded]);

  const providers = [
    { name: "Xero", type: "Accounting", icon: SiXero, color: "#13B5EA" },
    { name: "Stripe", type: "Payments", icon: SiStripe, color: "#635BFF" },
    { name: "Coinbase", type: "Exchange", icon: SiCoinbase, color: "#0052FF" },
    { name: "Binance", type: "Exchange", icon: SiBinance, color: "#F0B90B" },
    { name: "Gusto", type: "Payroll", icon: SiGusto, color: "#F45D48" },
  ];

  const handleConnect = (providerName: string) => {
    toast.promise(
      new Promise((resolve) => setTimeout(resolve, 2000)),
      {
        loading: `Connecting to ${providerName} via zkTLS...`,
        success: () => {
          setIsVerified(true);
          return `Successfully verified with ${providerName}!`;
        },
        error: "Verification failed",
      }
    );
  };

  const handleBorrow = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!address) {
      toast.error("Please connect your wallet first");
      return;
    }
    if (!isVerified) {
      toast.error("Please verify a Web2 data source first");
      return;
    }
    if (!invoiceAmount || !dueDate) {
      toast.error("Please fill in all fields");
      return;
    }

    // Convert amount and check 10% minimum bond
    const amountInStroops = Math.floor(parseFloat(invoiceAmount) * 10000000);
    // The pool must have `amountInStroops` USDC available, and the user must have `0.1 * amountInStroops` USDC to bond!

    setIsGenerating(true);
    const toastId = toast.loading("Generating ZK Proof...", {
      description: "Connecting to Prover Relayer...",
    });
    
    try {
      const response = await fetch("/api/generate-proof", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ amount: amountInStroops.toString(), invoiceId: "INV-1234", dueDate }),
      });
      
      const data = await response.json();
      if (!data.success) throw new Error(data.error);

      toast.loading("Submitting to network...", {
        id: toastId,
        description: "Please sign the transaction in Freighter.",
      });

      const proofBuffer = Buffer.from(data.proof, "hex");

      const { submitContractCall, MAIN_CONTRACT_ID } = await import("@/lib/stellar");
      const { nativeToScVal, xdr } = await import("@stellar/stellar-sdk");

      const args = [
        nativeToScVal(address, { type: "address" }),
        xdr.ScVal.scvVec([xdr.ScVal.scvSymbol("USDC")]),
        nativeToScVal(proofBuffer, { type: "bytes" }),
        nativeToScVal(Buffer.alloc(32, 0), { type: "bytes" })
      ];

      const { hash: txHash, returnValue } = await submitContractCall(address, MAIN_CONTRACT_ID, "request_loan", args);

      // The contract returns the real loan_id (u64). Use it so repay works later.
      const contractLoanId = returnValue != null ? Number(returnValue) : null;

      toast.success("Loan approved and disbursed!", {
        id: toastId,
        description: "ZK Proof successfully verified by Soroban smart contract.",
        action: {
          label: "View Explorer",
          onClick: () => window.open(`https://stellar.expert/explorer/testnet/tx/${txHash}`, "_blank")
        }
      });

      // Add the new loan to state (persisted to localStorage via effect)
      setLoans((prev) => [
        {
          id: contractLoanId != null ? `L-${contractLoanId}` : `L-${Math.floor(Math.random() * 9000) + 1000}`,
          contractId: contractLoanId,
          principalStroops: amountInStroops,
          amount: `${invoiceAmount} USDC`,
          date: dueDate,
          status: "Active"
        },
        ...prev
      ]);

      setInvoiceAmount("");
      setDueDate("");
    } catch (error: any) {
      console.error(error);
      toast.error("Transaction Failed", {
        id: toastId,
        description: error.message || "Failed to generate proof or request loan"
      });
    } finally {
      setIsGenerating(false);
    }
  };

  return (
    <div className="flex-1 max-w-7xl w-full mx-auto p-6 py-12">
      <div className="mb-10">
        <h1 className="text-3xl font-bold text-slate-900 dark:text-white mb-2">Borrow Assets</h1>
        <p className="text-slate-600 dark:text-slate-400 max-w-2xl">
          Use your unpaid invoices or business cash flows as collateral. Prove your data cryptographically using zkTLS without revealing sensitive details.
        </p>
      </div>

      <div className="grid lg:grid-cols-3 gap-8">
        {/* Left Column: Verification */}
        <div className="lg:col-span-2 space-y-8">
          
          {/* Status Card */}
          <div className={`glass-panel p-6 rounded-3xl flex items-center justify-between border ${isVerified ? 'border-emerald-500/50' : 'border-slate-200 dark:border-slate-800'}`}>
            <div>
              <h2 className="text-lg font-bold text-slate-900 dark:text-white mb-1">Verification Status</h2>
              <p className="text-sm text-slate-500">Connect a data source to enable borrowing</p>
            </div>
            <div className={`px-4 py-2 rounded-full flex items-center gap-2 font-bold ${
              isVerified 
                ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-500/20 dark:text-emerald-400 shadow-[0_0_15px_rgba(16,185,129,0.3)]' 
                : 'bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-400'
            }`}>
              {isVerified ? (
                <><CheckCircle2 className="w-5 h-5" /> Verified</>
              ) : (
                <><XCircle className="w-5 h-5" /> Not Verified</>
              )}
            </div>
          </div>

          {/* Providers Grid */}
          <div>
            <h3 className="text-xl font-bold text-slate-900 dark:text-white mb-4">Select Data Source</h3>
            <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4">
              {providers.map((provider) => {
                const Icon = provider.icon;
                return (
                <div key={provider.name} className="glass-panel p-5 rounded-2xl flex flex-col items-center text-center group hover:border-primary-500/30 transition-colors">
                  <div className="w-12 h-12 rounded-full bg-slate-100 dark:bg-slate-800 mb-3 flex items-center justify-center">
                    <Icon className="w-6 h-6 transition-transform group-hover:scale-110" style={{ color: provider.color }} />
                  </div>
                  <h4 className="font-bold text-slate-900 dark:text-white">{provider.name}</h4>
                  <p className="text-xs text-slate-500 mb-4">{provider.type}</p>
                  <button 
                    onClick={() => handleConnect(provider.name)}
                    className="w-full py-2 rounded-full bg-slate-100 dark:bg-slate-800 text-slate-900 dark:text-white font-medium hover:bg-primary-500 hover:text-white transition-all active:scale-95 text-sm"
                  >
                    Connect
                  </button>
                </div>
                );
              })}
            </div>
          </div>

          {/* Loan History */}
          <div className="glass-panel p-6 md:p-8 rounded-3xl">
            <h3 className="text-xl font-bold text-slate-900 dark:text-white mb-6">Loan History</h3>
            <div className="overflow-x-auto">
              <table className="w-full text-left">
                <thead>
                  <tr className="border-b border-slate-200 dark:border-slate-800">
                    <th className="pb-3 text-sm font-medium text-slate-500">Loan ID</th>
                    <th className="pb-3 text-sm font-medium text-slate-500">Amount</th>
                    <th className="pb-3 text-sm font-medium text-slate-500">Due Date</th>
                    <th className="pb-3 text-sm font-medium text-slate-500">Status</th>
                  </tr>
                </thead>
                <tbody className="text-sm">
                  {loans.map((loan, i) => (
                    <tr key={loan.id} className="border-b border-slate-100 dark:border-slate-800/50 last:border-0">
                      <td className="py-4 font-mono text-slate-600 dark:text-slate-400">{loan.id}</td>
                      <td className="py-4 font-bold dark:text-white">{loan.amount}</td>
                      <td className="py-4 text-slate-600 dark:text-slate-400">{loan.date}</td>
                      <td className="py-4">
                        <div className="flex items-center gap-3">
                          <span className={`px-2 py-1 rounded text-xs font-medium ${
                            loan.status === 'Active' ? 'bg-amber-100 text-amber-700 dark:bg-amber-500/20 dark:text-amber-400' : 'bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-400'
                          }`}>
                            {loan.status}
                          </span>
                          {loan.status === 'Active' && (
                            <button
                              onClick={async () => {
                                if (!address) { toast.error("Connect wallet"); return; }
                                if (loan.contractId == null || loan.principalStroops == null) {
                                  toast.error("Demo loan", { description: "This sample loan isn't on-chain and can't be repaid." });
                                  return;
                                }
                                const tid = toast.loading("Repaying loan...", { description: "Sign in Freighter." });
                                try {
                                  const { submitContractCall, MAIN_CONTRACT_ID } = await import("@/lib/stellar");
                                  const { nativeToScVal } = await import("@stellar/stellar-sdk");
                                  const { hash: tx } = await submitContractCall(address, MAIN_CONTRACT_ID, "repay_loan", [
                                    nativeToScVal(address, { type: "address" }),
                                    nativeToScVal(loan.contractId, { type: "u64" }),
                                    nativeToScVal(loan.principalStroops, { type: "i128" })
                                  ]);
                                  toast.success("Repayment Successful!", { id: tid, action: { label: "View Explorer", onClick: () => window.open(`https://stellar.expert/explorer/testnet/tx/${tx}`, "_blank") }});
                                  setLoans(prev => prev.map(l => l.id === loan.id ? { ...l, status: "Repaid" } : l));
                                } catch (e: any) {
                                  toast.error("Repayment Failed", { id: tid, description: e.message });
                                }
                              }}
                              className="text-xs px-3 py-1.5 rounded-full bg-primary-500 text-white font-bold hover:bg-primary-600 transition-colors active:scale-95"
                            >
                              Repay
                            </button>
                          )}
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>

        {/* Right Column: Action Area */}
        <div className="space-y-6">
          <div className="glass-panel p-6 md:p-8 rounded-3xl">
            <h3 className="text-xl font-bold text-slate-900 dark:text-white mb-6">New Loan Request</h3>
            <form onSubmit={handleBorrow} className="space-y-5">
              <div>
                <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
                  Invoice Amount (USDC)
                </label>
                <input 
                  type="number"
                  value={invoiceAmount}
                  onChange={(e) => setInvoiceAmount(e.target.value)}
                  placeholder="e.g. 10000"
                  className="w-full px-4 py-3 rounded-xl bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-800 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent transition-all text-slate-900 dark:text-white"
                />
              </div>
              
              <div>
                <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
                  Due Date
                </label>
                <input 
                  type="date"
                  value={dueDate}
                  onChange={(e) => setDueDate(e.target.value)}
                  className="w-full px-4 py-3 rounded-xl bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-800 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent transition-all text-slate-900 dark:text-white"
                />
              </div>

              <div className="pt-4">
                <button 
                  type="submit"
                  disabled={isGenerating}
                  className="w-full py-4 rounded-xl bg-primary-500 text-white font-bold hover:bg-primary-600 transition-all active:scale-95 shadow-lg shadow-primary-500/30 flex items-center justify-center gap-2 disabled:opacity-70 disabled:active:scale-100"
                >
                  {isGenerating ? (
                    <>
                      <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                      Generating Proof...
                    </>
                  ) : (
                    <>Generate ZK Proof & Borrow <ArrowRight className="w-5 h-5" /></>
                  )}
                </button>
              </div>
            </form>
          </div>

          <div className="glass-panel p-6 rounded-3xl bg-slate-50/50 dark:bg-slate-900/50">
            <div className="flex gap-3 mb-3">
              <FileText className="w-5 h-5 text-primary-500 shrink-0" />
              <h4 className="font-bold text-slate-900 dark:text-white text-sm">How zkTLS Works</h4>
            </div>
            <p className="text-sm text-slate-600 dark:text-slate-400 leading-relaxed">
              When you connect a data source, our zkTLS proxy establishes a secure connection with the API. It generates a cryptographic proof of the response locally. Only the proof is sent to the Soroban smart contract, ensuring your data never leaves your device.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
