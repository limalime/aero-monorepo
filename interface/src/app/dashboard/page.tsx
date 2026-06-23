"use client";

import { useState, useEffect } from "react";
import { useWallet } from "@/hooks/use-wallet";
import { Activity, ArrowDownRight, ArrowUpRight, DollarSign, Percent, PieChart, ShieldAlert } from "lucide-react";
import { readContract, MAIN_CONTRACT_ID } from "@/lib/stellar";
import { nativeToScVal } from "@stellar/stellar-sdk";

export default function DashboardPage() {
  const { address } = useWallet();

  const [isLoading, setIsLoading] = useState(true);
  const [tvl, setTvl] = useState("0");
  const [apr, setApr] = useState("0");
  const [activeLoans, setActiveLoans] = useState("0");

  useEffect(() => {
    async function fetchDashboardData() {
      if (!address) return;
      try {
        setIsLoading(true);
        // Note: Replace "get_user_stats" with the exact Soroban contract method
        // Using readContract from our stellar lib
        // const stats = await readContract(MAIN_CONTRACT_ID, "get_user_stats", [nativeToScVal(address, { type: "address" })]);
        
        // Mocking the network delay and response for now since contracts might be custom
        await new Promise(r => setTimeout(r, 1500));
        
        setTvl("$45,230.00");
        setApr("8.4%");
        setActiveLoans("2");
      } catch (error) {
        console.error("Failed to fetch dashboard data:", error);
      } finally {
        setIsLoading(false);
      }
    }
    
    fetchDashboardData();
  }, [address]);

  const metrics = [
    { label: "Total Supplied (TVL)", value: tvl, icon: <DollarSign className="text-primary-500" />, trend: "+2.4%" },
    { label: "Average APR", value: apr, icon: <Percent className="text-primary-500" />, trend: "+0.2%" },
    { label: "Active Loans", value: activeLoans, icon: <Activity className="text-primary-500" />, trend: "Stable" },
    { label: "Health Factor", value: "2.4", icon: <ShieldAlert className="text-emerald-500" />, trend: "Safe", color: "text-emerald-500" },
  ];

  const activities = [
    { type: "Deposit", amount: "+5,000 USDC", date: "Today, 14:30", hash: "0x12...34" },
    { type: "Repayment", amount: "-1,200 USDC", date: "Yesterday, 09:15", hash: "0x56...78" },
    { type: "Borrow", amount: "+10,000 XLM", date: "Jun 15, 11:20", hash: "0x9a...bc" },
    { type: "Deposit", amount: "+2,000 XLM", date: "Jun 10, 16:45", hash: "0xde...f0" },
  ];

  if (!address) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center p-6 text-center">
        <div className="w-24 h-24 rounded-full glass-panel flex items-center justify-center mb-6">
          <Activity className="w-10 h-10 text-slate-400" />
        </div>
        <h2 className="text-2xl font-bold text-slate-900 dark:text-white mb-2">Wallet Not Connected</h2>
        <p className="text-slate-600 dark:text-slate-400 max-w-md">Please connect your Freighter wallet to view your dashboard, manage positions, and track activity.</p>
      </div>
    );
  }

  return (
    <div className="flex-1 max-w-7xl w-full mx-auto p-6 py-12">
      <div className="mb-10">
        <h1 className="text-3xl font-bold text-slate-900 dark:text-white mb-2">Dashboard</h1>
        <p className="text-slate-600 dark:text-slate-400">Welcome back. Here's an overview of your Aero protocol positions.</p>
      </div>

      {/* Metrics Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-10">
        {metrics.map((m, i) => (
          <div key={i} className="glass-panel p-6 rounded-3xl flex flex-col">
            <div className="flex items-center justify-between mb-4">
              <div className="w-10 h-10 rounded-full bg-slate-100 dark:bg-slate-800 flex items-center justify-center">
                {m.icon}
              </div>
              <span className="text-xs font-medium px-2 py-1 bg-slate-100 dark:bg-slate-800 rounded-full text-slate-600 dark:text-slate-400">
                {isLoading ? <div className="w-8 h-3 rounded bg-slate-200 dark:bg-slate-700 animate-pulse" /> : m.trend}
              </span>
            </div>
            <div className="text-sm font-medium text-slate-500 dark:text-slate-400 mb-1">{m.label}</div>
            {isLoading ? (
              <div className="h-8 w-24 rounded-lg bg-slate-200 dark:bg-slate-800 animate-pulse mt-1" />
            ) : (
              <div className={`text-2xl font-bold ${m.color ? m.color : "text-slate-900 dark:text-white"}`}>{m.value}</div>
            )}
          </div>
        ))}
      </div>

      <div className="grid lg:grid-cols-3 gap-8">
        {/* Portfolio Breakdown */}
        <div className="glass-panel p-6 md:p-8 rounded-3xl lg:col-span-1 flex flex-col">
          <h3 className="text-xl font-bold text-slate-900 dark:text-white mb-6 flex items-center gap-2">
            <PieChart className="w-5 h-5 text-primary-500" /> Portfolio Breakdown
          </h3>
          <div className="flex-1 flex flex-col items-center justify-center py-6">
            {isLoading ? (
              <div className="w-48 h-48 rounded-full bg-slate-200 dark:bg-slate-800 animate-pulse mb-8" />
            ) : (
              <div className="w-48 h-48 rounded-full bg-gradient-to-tr from-primary-600 to-primary-400 relative mb-8 shadow-inner border-[8px] border-slate-50 dark:border-slate-900 flex items-center justify-center">
                 <div className="absolute inset-0 rounded-full border-[8px] border-emerald-400" style={{ clipPath: "polygon(50% 50%, 100% 0, 100% 100%, 50% 100%)" }} />
                 <div className="w-32 h-32 rounded-full bg-slate-50 dark:bg-slate-950 flex items-center justify-center flex-col z-10 shadow-lg">
                    <span className="text-sm text-slate-500">Total</span>
                    <span className="font-bold text-lg dark:text-white">{tvl}</span>
                 </div>
              </div>
            )}

            <div className="w-full space-y-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 rounded-full bg-primary-500" />
                  <span className="text-slate-700 dark:text-slate-300 font-medium">USDC</span>
                </div>
                <div className="font-bold dark:text-white">65%</div>
              </div>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 rounded-full bg-emerald-400" />
                  <span className="text-slate-700 dark:text-slate-300 font-medium">XLM</span>
                </div>
                <div className="font-bold dark:text-white">35%</div>
              </div>
            </div>
          </div>
        </div>

        {/* Recent Activity */}
        <div className="glass-panel p-6 md:p-8 rounded-3xl lg:col-span-2">
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-xl font-bold text-slate-900 dark:text-white">Recent Activity</h3>
            <button className="text-sm font-medium text-primary-500 hover:text-primary-600">View All</button>
          </div>
          
          <div className="space-y-4">
            {isLoading ? (
              Array.from({ length: 4 }).map((_, i) => (
                <div key={i} className="flex items-center justify-between p-4 rounded-2xl bg-slate-50/50 dark:bg-slate-800/30 border border-slate-200 dark:border-slate-800">
                  <div className="flex items-center gap-4">
                    <div className="w-10 h-10 rounded-full bg-slate-200 dark:bg-slate-700 animate-pulse" />
                    <div>
                      <div className="w-24 h-4 rounded bg-slate-200 dark:bg-slate-700 animate-pulse mb-2" />
                      <div className="w-16 h-3 rounded bg-slate-200 dark:bg-slate-700 animate-pulse" />
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="w-20 h-4 rounded bg-slate-200 dark:bg-slate-700 animate-pulse mb-2 ml-auto" />
                    <div className="w-16 h-3 rounded bg-slate-200 dark:bg-slate-700 animate-pulse ml-auto" />
                  </div>
                </div>
              ))
            ) : (
              activities.map((act, i) => {
                const isPositive = act.amount.startsWith("+");
                return (
                  <div key={i} className="flex items-center justify-between p-4 rounded-2xl bg-slate-50/50 dark:bg-slate-800/30 hover:bg-slate-100 dark:hover:bg-slate-800/60 transition-colors border border-transparent hover:border-slate-200 dark:hover:border-slate-700">
                    <div className="flex items-center gap-4">
                      <div className={`w-10 h-10 rounded-full flex items-center justify-center ${isPositive ? 'bg-emerald-100 text-emerald-600 dark:bg-emerald-500/20 dark:text-emerald-400' : 'bg-rose-100 text-rose-600 dark:bg-rose-500/20 dark:text-rose-400'}`}>
                        {isPositive ? <ArrowDownRight className="w-5 h-5" /> : <ArrowUpRight className="w-5 h-5" />}
                      </div>
                      <div>
                        <div className="font-bold text-slate-900 dark:text-white">{act.type}</div>
                        <div className="text-sm text-slate-500">{act.date}</div>
                      </div>
                    </div>
                    <div className="text-right">
                      <div className={`font-bold ${isPositive ? 'text-emerald-600 dark:text-emerald-400' : 'text-slate-900 dark:text-white'}`}>
                        {act.amount}
                      </div>
                      <div className="text-sm text-slate-500 font-mono">Tx: {act.hash}</div>
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
