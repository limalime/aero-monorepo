"use client";

import { useState, useEffect } from "react";
import { VaultModal } from "@/components/vault-modal";
import { DollarSign, Droplets, Info } from "lucide-react";
import { useWallet } from "@/hooks/use-wallet";
import { readContract, submitContractCall, MAIN_CONTRACT_ID } from "@/lib/stellar";
import { nativeToScVal, xdr } from "@stellar/stellar-sdk";
import { toast } from "sonner";

export default function MarketsPage() {
  const { address } = useWallet();
  const [activeTab, setActiveTab] = useState<"open" | "closed">("open");
  const [selectedVault, setSelectedVault] = useState<any | null>(null);
  
  const [isLoading, setIsLoading] = useState(true);
  const [globalStats, setGlobalStats] = useState({ volume: "0", xlm: "0", usdc: "0" });
  const [vaults, setVaults] = useState<any[]>([]);

  useEffect(() => {
    async function fetchMarketsData() {
      try {
        setIsLoading(true);
        // Mocking network delay and actual fetching
        // const stats = await readContract(MAIN_CONTRACT_ID, "get_pool_stats", []);
        await new Promise(r => setTimeout(r, 1500));
        
        setGlobalStats({
          volume: "$12,450,000",
          xlm: "4,200,000",
          usdc: "8,250,000"
        });

        setVaults([
          {
            id: 1,
            name: "SME Invoice Pool Alpha",
            apr: "8.5%",
            capacity: "$2.5M / $5M",
            address: MAIN_CONTRACT_ID,
            asset: "USDC",
            status: "Healthy",
            type: "open"
          },
          {
            id: 2,
            name: "Tech Founders Factoring",
            apr: "10.2%",
            capacity: "$1.2M / $2M",
            address: MAIN_CONTRACT_ID,
            asset: "USDC",
            status: "Healthy",
            type: "open"
          },
          {
            id: 3,
            name: "Retail Merchants XLM",
            apr: "6.8%",
            capacity: "500K / 1M XLM",
            address: MAIN_CONTRACT_ID,
            asset: "XLM",
            status: "Pending",
            type: "open"
          },
          {
            id: 4,
            name: "Construction Supply Q1",
            apr: "12.0%",
            capacity: "$4M / $4M",
            address: MAIN_CONTRACT_ID,
            asset: "USDC",
            status: "Closed",
            type: "closed"
          }
        ]);
      } catch (error) {
        console.error("Failed to fetch markets data:", error);
        toast.error("Failed to fetch markets data from network");
      } finally {
        setIsLoading(false);
      }
    }
    
    fetchMarketsData();
  }, []);

  const displayVaults = vaults.filter(v => v.type === activeTab);

  return (
    <div className="flex-1 max-w-7xl w-full mx-auto p-6 py-12">
      <div className="flex flex-col md:flex-row justify-between items-start md:items-end mb-10 gap-6">
        <div>
          <h1 className="text-3xl font-bold text-slate-900 dark:text-white mb-2">Liquidity Markets</h1>
          <p className="text-slate-600 dark:text-slate-400 max-w-2xl">
            Supply assets to decentralized factoring pools. Your capital is lent to verified businesses backed by ZK proofs.
          </p>
        </div>
        <div className="flex gap-4">
          <button 
            onClick={() => toast("Vault Creation Restricted", { description: "Only verified institutions can create new liquidity vaults." })}
            className="px-6 py-3 rounded-full bg-slate-900 dark:bg-white text-white dark:text-slate-900 font-bold hover:opacity-90 transition-opacity active:scale-95 shadow-lg"
          >
            Make Vaults
          </button>
        </div>
      </div>

      {/* Global Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-10">
        <div className="glass-panel p-6 rounded-3xl">
          <div className="text-sm font-medium text-slate-500 mb-1 flex items-center gap-2"><DollarSign className="w-4 h-4"/> Total Markets Volume</div>
          {isLoading ? <div className="h-9 w-32 rounded-lg bg-slate-200 dark:bg-slate-800 animate-pulse mt-1" /> : <div className="text-3xl font-bold dark:text-white">{globalStats.volume}</div>}
        </div>
        <div className="glass-panel p-6 rounded-3xl">
          <div className="text-sm font-medium text-slate-500 mb-1 flex items-center gap-2"><Droplets className="w-4 h-4"/> XLM Supplied</div>
          {isLoading ? <div className="h-9 w-32 rounded-lg bg-slate-200 dark:bg-slate-800 animate-pulse mt-1" /> : <div className="text-3xl font-bold dark:text-white">{globalStats.xlm}</div>}
        </div>
        <div className="glass-panel p-6 rounded-3xl">
          <div className="text-sm font-medium text-slate-500 mb-1 flex items-center gap-2"><Droplets className="w-4 h-4 text-primary-500"/> USDC Supplied</div>
          {isLoading ? <div className="h-9 w-32 rounded-lg bg-slate-200 dark:bg-slate-800 animate-pulse mt-1" /> : <div className="text-3xl font-bold text-primary-500">{globalStats.usdc}</div>}
        </div>
      </div>

      {/* Tabs */}
      <div className="flex items-center gap-2 mb-8 border-b border-slate-200 dark:border-slate-800 pb-px">
        <button 
          onClick={() => setActiveTab("open")}
          className={`px-6 py-3 font-medium transition-colors border-b-2 ${activeTab === 'open' ? 'border-primary-500 text-primary-500' : 'border-transparent text-slate-500 hover:text-slate-700 dark:hover:text-slate-300'}`}
        >
          Open Markets
        </button>
        <button 
          onClick={() => setActiveTab("closed")}
          className={`px-6 py-3 font-medium transition-colors border-b-2 ${activeTab === 'closed' ? 'border-primary-500 text-primary-500' : 'border-transparent text-slate-500 hover:text-slate-700 dark:hover:text-slate-300'}`}
        >
          Closed Markets
        </button>
      </div>

      {/* Vault Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {isLoading ? (
          Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="glass-panel rounded-3xl p-6 flex flex-col">
              <div className="flex justify-between items-start mb-6">
                <div className="w-12 h-12 rounded-full bg-slate-200 dark:bg-slate-800 animate-pulse" />
                <div className="w-16 h-6 rounded-full bg-slate-200 dark:bg-slate-800 animate-pulse" />
              </div>
              <div className="w-3/4 h-6 rounded bg-slate-200 dark:bg-slate-800 animate-pulse mb-2" />
              <div className="w-1/4 h-4 rounded bg-slate-200 dark:bg-slate-800 animate-pulse mb-6" />
              <div className="grid grid-cols-2 gap-4 mb-8">
                <div>
                  <div className="w-16 h-3 rounded bg-slate-200 dark:bg-slate-800 animate-pulse mb-2" />
                  <div className="w-12 h-5 rounded bg-slate-200 dark:bg-slate-800 animate-pulse" />
                </div>
                <div>
                  <div className="w-16 h-3 rounded bg-slate-200 dark:bg-slate-800 animate-pulse mb-2" />
                  <div className="w-20 h-5 rounded bg-slate-200 dark:bg-slate-800 animate-pulse" />
                </div>
              </div>
              <div className="mt-auto flex gap-3">
                <div className="flex-1 h-10 rounded-full bg-slate-200 dark:bg-slate-800 animate-pulse" />
                <div className="w-12 h-10 rounded-full bg-slate-200 dark:bg-slate-800 animate-pulse" />
              </div>
            </div>
          ))
        ) : (
          displayVaults.map((vault) => (
            <div key={vault.id} className="glass-panel rounded-3xl p-6 flex flex-col group hover:border-primary-500/50 transition-all">
              <div className="flex justify-between items-start mb-6">
                <div className="w-12 h-12 rounded-full bg-primary-500/10 flex items-center justify-center text-primary-500 font-bold text-xl">
                  {vault.asset === 'USDC' ? '$' : 'X'}
                </div>
                <div className={`px-3 py-1 text-xs font-bold rounded-full ${
                  vault.status === 'Healthy' ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-500/20 dark:text-emerald-400' :
                  vault.status === 'Pending' ? 'bg-amber-100 text-amber-700 dark:bg-amber-500/20 dark:text-amber-400' :
                  'bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-400'
                }`}>
                  {vault.status}
                </div>
              </div>
              <h3 className="text-xl font-bold text-slate-900 dark:text-white mb-1">{vault.name}</h3>
              <p className="text-sm text-slate-500 dark:text-slate-400 mb-6">Asset: {vault.asset}</p>
              
              <div className="grid grid-cols-2 gap-4 mb-8">
                <div>
                  <div className="text-xs text-slate-500 mb-1">Target APR</div>
                  <div className="text-lg font-bold text-primary-500">{vault.apr}</div>
                </div>
                <div>
                  <div className="text-xs text-slate-500 mb-1">Capacity</div>
                  <div className="text-sm font-bold dark:text-white pt-1">{vault.capacity}</div>
                </div>
              </div>

              <div className="mt-auto flex gap-3">
                <button 
                  disabled={vault.type === 'closed'}
                  onClick={() => setSelectedVault(vault)}
                  className="flex-1 py-2.5 rounded-full bg-primary-500 text-white font-medium hover:bg-primary-600 transition-colors active:scale-95 disabled:opacity-50 disabled:active:scale-100"
                >
                  Deposit
                </button>
                <button 
                  onClick={() => setSelectedVault(vault)}
                  className="px-4 py-2.5 rounded-full glass-panel text-slate-700 dark:text-slate-300 font-medium hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors active:scale-95 flex items-center justify-center"
                  aria-label="More details"
                >
                  <Info className="w-5 h-5" />
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      <VaultModal 
        isOpen={!!selectedVault} 
        closeModal={() => setSelectedVault(null)} 
        vault={selectedVault} 
      />
    </div>
  );
}
