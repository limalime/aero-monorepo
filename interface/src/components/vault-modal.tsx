"use client";

import { Dialog, Transition } from "@headlessui/react";
import { Fragment, useState } from "react";
import { X, ExternalLink, TrendingUp, ShieldAlert } from "lucide-react";
import { useWallet } from "@/hooks/use-wallet";
import { submitContractCall } from "@/lib/stellar";
import { nativeToScVal, xdr } from "@stellar/stellar-sdk";
import { toast } from "sonner";

interface VaultModalProps {
  isOpen: boolean;
  closeModal: () => void;
  vault: {
    name: string;
    apr: string;
    capacity: string;
    address: string;
    asset: string;
    status: string;
  } | null;
}

export function VaultModal({ isOpen, closeModal, vault }: VaultModalProps) {
  const { address } = useWallet();
  const [amountInput, setAmountInput] = useState("1000");

  const handleTransaction = async (action: "deposit" | "withdraw") => {
    if (!address || !vault) {
      toast.error("Please connect your wallet first");
      return;
    }

    const amount = parseFloat(amountInput);
    if (isNaN(amount) || amount < 1000) {
      toast.error("Invalid Amount", { description: "Minimum amount is 1000" });
      return;
    }

    const toastId = toast.loading("Transaction Pending...", {
      description: "Please sign the transaction in Freighter.",
    });

    try {
      // 7 decimals for Soroban
      const amountInStroops = Math.floor(amount * 10000000);
      const args = [
        nativeToScVal(address, { type: "address" }),
        xdr.ScVal.scvVec([xdr.ScVal.scvSymbol(vault.asset)]),
        nativeToScVal(amountInStroops, { type: "i128" })
      ];

      const { hash: txHash } = await submitContractCall(address, vault.address, action, args);
      
      toast.success(`${action === "deposit" ? "Deposit" : "Withdraw"} Successful!`, {
        id: toastId,
        description: `Your assets have been ${action === "deposit" ? "supplied to" : "withdrawn from"} the pool.`,
        action: {
          label: "View Explorer",
          onClick: () => window.open(`https://stellar.expert/explorer/testnet/tx/${txHash}`, "_blank")
        }
      });
      closeModal();
    } catch (error: any) {
      console.error(error);
      toast.error("Transaction Rejected", {
        id: toastId,
        description: error.message || `Failed to submit ${action} transaction.`,
      });
    }
  };

  if (!vault) return null;

  return (
    <Transition appear show={isOpen} as={Fragment}>
      <Dialog as="div" className="relative z-[100]" onClose={closeModal}>
        <Transition.Child
          as={Fragment}
          enter="ease-out duration-300"
          enterFrom="opacity-0"
          enterTo="opacity-100"
          leave="ease-in duration-200"
          leaveFrom="opacity-100"
          leaveTo="opacity-0"
        >
          <div className="fixed inset-0 bg-slate-900/40 backdrop-blur-sm" />
        </Transition.Child>

        <div className="fixed inset-0 overflow-y-auto">
          <div className="flex min-h-full items-center justify-center p-4 text-center">
            <Transition.Child
              as={Fragment}
              enter="ease-out duration-300"
              enterFrom="opacity-0 scale-95 translate-y-4"
              enterTo="opacity-100 scale-100 translate-y-0"
              leave="ease-in duration-200"
              leaveFrom="opacity-100 scale-100 translate-y-0"
              leaveTo="opacity-0 scale-95 translate-y-4"
            >
              <Dialog.Panel className="w-full max-w-lg transform overflow-hidden rounded-3xl glass-modal p-6 text-left align-middle shadow-2xl transition-all">
                <div className="flex items-center justify-between mb-6">
                  <Dialog.Title as="h3" className="text-2xl font-bold leading-6 text-slate-900 dark:text-white">
                    {vault.name} Details
                  </Dialog.Title>
                  <button
                    onClick={closeModal}
                    className="p-2 rounded-full hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors active:scale-95"
                  >
                    <X className="w-5 h-5 text-slate-500" />
                  </button>
                </div>

                <div className="space-y-6">
                  <div className="grid grid-cols-2 gap-4">
                    <div className="p-4 rounded-2xl bg-slate-50 dark:bg-slate-800/50">
                      <div className="text-sm text-slate-500 mb-1 flex items-center gap-2"><TrendingUp className="w-4 h-4" /> Current APR</div>
                      <div className="text-xl font-bold text-primary-500">{vault.apr}</div>
                    </div>
                    <div className="p-4 rounded-2xl bg-slate-50 dark:bg-slate-800/50">
                      <div className="text-sm text-slate-500 mb-1 flex items-center gap-2"><ShieldAlert className="w-4 h-4" /> Capacity</div>
                      <div className="text-xl font-bold dark:text-white">{vault.capacity}</div>
                    </div>
                  </div>

                  <div>
                    <h4 className="text-sm font-bold text-slate-900 dark:text-white mb-2">Amount to supply/withdraw</h4>
                    <div className="relative">
                      <div className="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none text-slate-500 font-bold">
                        {vault.asset === 'USDC' ? '$' : 'X'}
                      </div>
                      <input 
                        type="number"
                        value={amountInput}
                        onChange={(e) => setAmountInput(e.target.value)}
                        placeholder="1000"
                        min="1000"
                        className="w-full bg-slate-100 dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl py-3 pl-8 pr-16 focus:outline-none focus:ring-2 focus:ring-primary-500/50 text-slate-900 dark:text-white font-bold"
                      />
                      <div className="absolute inset-y-0 right-0 pr-4 flex items-center pointer-events-none text-slate-500 font-bold">
                        {vault.asset}
                      </div>
                    </div>
                  </div>

                  <div>
                    <h4 className="text-sm font-bold text-slate-900 dark:text-white mb-2">Smart Contract Address</h4>
                    <div className="flex items-center justify-between p-3 rounded-xl bg-slate-100 dark:bg-slate-900 border border-slate-200 dark:border-slate-800 font-mono text-sm text-slate-600 dark:text-slate-400">
                      <span>{vault.address}</span>
                      <button className="text-primary-500 hover:text-primary-600 p-1">
                        <ExternalLink className="w-4 h-4" />
                      </button>
                    </div>
                  </div>

                  <div>
                    <h4 className="text-sm font-bold text-slate-900 dark:text-white mb-3">Historical Yield (30D)</h4>
                    <div className="h-40 w-full rounded-xl bg-slate-50 dark:bg-slate-800/50 flex items-end justify-between p-4 border border-slate-100 dark:border-slate-800">
                      {/* Simple CSS Chart */}
                      {[30, 45, 40, 55, 60, 50, 75, 80, 70, 85, 95, 90].map((h, i) => (
                        <div key={i} className="w-[6%] bg-primary-500/80 rounded-t-sm hover:bg-primary-400 transition-colors cursor-pointer group relative" style={{ height: `${h}%` }}>
                          <div className="absolute -top-8 left-1/2 -translate-x-1/2 bg-slate-900 text-white text-xs py-1 px-2 rounded opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity whitespace-nowrap z-10">
                            {((h/100) * 12).toFixed(1)}%
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>

                <div className="mt-8 flex gap-4">
                  <button 
                    onClick={() => handleTransaction("withdraw")}
                    className="flex-1 py-3 rounded-full bg-slate-100 dark:bg-slate-800 text-slate-900 dark:text-white font-bold hover:bg-slate-200 dark:hover:bg-slate-700 transition-all active:scale-95"
                  >
                    Withdraw
                  </button>
                  <button 
                    onClick={() => handleTransaction("deposit")}
                    className="flex-1 py-3 rounded-full bg-primary-500 text-white font-bold hover:bg-primary-600 transition-all active:scale-95 shadow-lg shadow-primary-500/30"
                  >
                    Deposit
                  </button>
                </div>
              </Dialog.Panel>
            </Transition.Child>
          </div>
        </div>
      </Dialog>
    </Transition>
  );
}
