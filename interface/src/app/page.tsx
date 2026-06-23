import Link from "next/link";
import { ArrowRight, ShieldCheck, Zap, Lock, FileText, CheckCircle2 } from "lucide-react";

export default function LandingPage() {
  return (
    <div className="flex flex-col flex-1">
      {/* Hero Section */}
      <section className="relative pt-32 pb-20 px-6 overflow-hidden">
        <div className="max-w-7xl mx-auto text-center relative z-10">
          <div data-aos="fade-up" className="inline-flex items-center gap-2 px-4 py-2 rounded-full glass-panel mb-8 border-primary-500/30">
            <span className="w-2 h-2 rounded-full bg-primary-500 animate-pulse" />
            <span className="text-sm font-medium text-slate-800 dark:text-slate-200">Aero is live on Stellar Testnet</span>
          </div>
          
          <h1 data-aos="fade-up" data-aos-delay="100" className="text-5xl md:text-7xl font-bold tracking-tight text-slate-900 dark:text-white mb-8 leading-tight">
            Zero-Knowledge <br />
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-primary-400 to-primary-600">
              Trade Finance
            </span>
          </h1>
          
          <p data-aos="fade-up" data-aos-delay="200" className="text-xl md:text-2xl text-slate-600 dark:text-slate-300 max-w-3xl mx-auto mb-10 leading-relaxed">
            Borrow against unpaid real-world invoices instantly. Keep your business data 100% private with ZK proofs on the Stellar network.
          </p>
          
          <div data-aos="fade-up" data-aos-delay="300" className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <Link 
              href="/dashboard"
              className="px-8 py-4 rounded-full bg-primary-500 text-white font-bold text-lg hover:bg-primary-600 transition-all active:scale-95 shadow-xl shadow-primary-500/30 flex items-center gap-2 w-full sm:w-auto justify-center"
            >
              Launch App <ArrowRight className="w-5 h-5" />
            </Link>
            <Link 
              href="#how-it-works"
              className="px-8 py-4 rounded-full glass-panel text-slate-900 dark:text-white font-bold text-lg transition-all active:scale-95 w-full sm:w-auto justify-center flex bg-transparent"
            >
              Learn More
            </Link>
          </div>
        </div>

        {/* Hero Mockup */}
        <div data-aos="fade-up" data-aos-delay="500" className="max-w-5xl mx-auto mt-20 relative">
          <div className="absolute inset-0 bg-gradient-to-t from-background to-transparent z-10 h-full w-full pointer-events-none translate-y-1/2" />
          <div className="glass-panel rounded-t-3xl border-b-0 p-4 md:p-8 relative overflow-hidden">
            <div className="flex items-center justify-between mb-8 pb-4 border-b border-slate-200 dark:border-slate-800">
              <div className="flex gap-4">
                <div className="w-12 h-12 rounded-xl bg-primary-500/20 flex items-center justify-center">
                  <ShieldCheck className="w-6 h-6 text-primary-500" />
                </div>
                <div>
                  <h3 className="font-bold text-slate-900 dark:text-white">Total Value Locked</h3>
                  <p className="text-2xl font-bold text-primary-500">$12,450,000</p>
                </div>
              </div>
              <div className="hidden md:flex gap-2">
                <div className="px-4 py-2 rounded-full bg-slate-100 dark:bg-slate-800 text-sm font-medium">USDC Market</div>
                <div className="px-4 py-2 rounded-full bg-slate-100 dark:bg-slate-800 text-sm font-medium">XLM Market</div>
              </div>
            </div>
            {/* Mock Chart Area */}
            <div className="h-48 w-full rounded-xl bg-gradient-to-r from-slate-100 to-slate-50 dark:from-slate-800 dark:to-slate-900 relative overflow-hidden flex items-end">
               {/* Decorative bars */}
               <div className="w-full flex items-end justify-between px-4 pb-4 gap-2 opacity-60">
                 {[40, 70, 45, 90, 65, 80, 100, 85, 60].map((h, i) => (
                   <div key={i} style={{ height: `${h}%` }} className="w-full bg-primary-500/40 rounded-t-sm relative">
                     <div className="absolute top-0 w-full h-1 bg-primary-500" />
                   </div>
                 ))}
               </div>
            </div>
          </div>
        </div>
      </section>

      {/* Products Detail */}
      <section className="py-24 px-6 relative z-20">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-16">
            <h2 data-aos="fade-up" className="text-3xl md:text-5xl font-bold text-slate-900 dark:text-white mb-6">Institutional Grade DeFi</h2>
            <p data-aos="fade-up" data-aos-delay="100" className="text-xl text-slate-600 dark:text-slate-400 max-w-2xl mx-auto">
              Aero merges traditional finance with Web3 efficiency, providing secure yields for lenders and instant liquidity for businesses.
            </p>
          </div>

          <div className="grid md:grid-cols-2 gap-8">
            <div data-aos="fade-up" data-aos-delay="200" className="glass-panel p-8 md:p-10 rounded-3xl relative overflow-hidden group hover:border-primary-500/50 transition-colors">
              <div className="absolute -right-20 -top-20 w-64 h-64 bg-primary-500/10 rounded-full blur-3xl group-hover:bg-primary-500/20 transition-colors" />
              <Zap className="w-12 h-12 text-primary-500 mb-6" />
              <h3 className="text-2xl font-bold text-slate-900 dark:text-white mb-4">Multi-Asset Liquidity Pools</h3>
              <p className="text-slate-600 dark:text-slate-300 mb-6 leading-relaxed">
                Supply XLM or USDC to our audited Soroban smart contracts. Earn sustainable, real-world yields generated from factored business invoices. Zero algorithmic stablecoin risk.
              </p>
              <ul className="space-y-3">
                <li className="flex items-center gap-3 text-slate-700 dark:text-slate-300">
                  <CheckCircle2 className="w-5 h-5 text-primary-500" /> Native USDC Integration
                </li>
                <li className="flex items-center gap-3 text-slate-700 dark:text-slate-300">
                  <CheckCircle2 className="w-5 h-5 text-primary-500" /> Auto-compounding Returns
                </li>
              </ul>
            </div>

            <div data-aos="fade-up" data-aos-delay="300" className="glass-panel p-8 md:p-10 rounded-3xl relative overflow-hidden group hover:border-primary-500/50 transition-colors">
              <div className="absolute -right-20 -top-20 w-64 h-64 bg-primary-500/10 rounded-full blur-3xl group-hover:bg-primary-500/20 transition-colors" />
              <Lock className="w-12 h-12 text-primary-500 mb-6" />
              <h3 className="text-2xl font-bold text-slate-900 dark:text-white mb-4">ZK-Verified RWA Loans</h3>
              <p className="text-slate-600 dark:text-slate-300 mb-6 leading-relaxed">
                Borrowers connect their accounting software via zkTLS. We generate a cryptographic proof of the invoice validity without ever seeing the sensitive data.
              </p>
              <ul className="space-y-3">
                <li className="flex items-center gap-3 text-slate-700 dark:text-slate-300">
                  <CheckCircle2 className="w-5 h-5 text-primary-500" /> 100% Data Privacy
                </li>
                <li className="flex items-center gap-3 text-slate-700 dark:text-slate-300">
                  <CheckCircle2 className="w-5 h-5 text-primary-500" /> Instant On-Chain Verification
                </li>
              </ul>
            </div>
          </div>
        </div>
      </section>

      {/* How It Works */}
      <section id="how-it-works" className="py-24 px-6 relative z-20 bg-slate-50 dark:bg-slate-900/20">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-16">
            <h2 data-aos="fade-up" className="text-3xl md:text-5xl font-bold text-slate-900 dark:text-white mb-6">How It Works</h2>
            <p data-aos="fade-up" data-aos-delay="100" className="text-xl text-slate-600 dark:text-slate-400 max-w-2xl mx-auto">
              Three simple steps to unlock your trapped capital.
            </p>
          </div>

          <div className="grid md:grid-cols-3 gap-8 relative">
            <div className="hidden md:block absolute top-1/2 left-[10%] right-[10%] h-0.5 bg-gradient-to-r from-transparent via-primary-500/30 to-transparent -translate-y-1/2 z-0" />
            
            {[
              {
                icon: <FileText className="w-8 h-8" />,
                step: "01",
                title: "Connect Web2 API",
                desc: "Securely connect your accounting software (Xero, Stripe) via our zkTLS proxy."
              },
              {
                icon: <ShieldCheck className="w-8 h-8" />,
                step: "02",
                title: "Generate ZK Proof",
                desc: "A Noir/RISC Zero proof is generated locally, validating your invoice without exposing data."
              },
              {
                icon: <Zap className="w-8 h-8" />,
                step: "03",
                title: "Get Instant Liquidity",
                desc: "The Soroban contract verifies the proof and instantly disburses USDC to your wallet."
              }
            ].map((item, i) => (
              <div key={i} data-aos="fade-up" data-aos-delay={200 + i * 100} className="relative z-10 flex flex-col items-center text-center">
                <div className="w-20 h-20 rounded-2xl glass-panel flex items-center justify-center text-primary-500 mb-6 rotate-3 hover:rotate-0 transition-transform">
                  {item.icon}
                </div>
                <div className="text-sm font-bold text-primary-500 mb-2 tracking-widest uppercase">Step {item.step}</div>
                <h4 className="text-xl font-bold text-slate-900 dark:text-white mb-3">{item.title}</h4>
                <p className="text-slate-600 dark:text-slate-400">{item.desc}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-32 px-6 relative z-20">
        <div className="max-w-4xl mx-auto text-center glass-panel p-12 md:p-20 rounded-[3rem] relative overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-br from-primary-500/10 to-transparent" />
          <h2 data-aos="fade-up" className="text-4xl md:text-5xl font-bold text-slate-900 dark:text-white mb-6 relative z-10">
            Ready to unlock your capital?
          </h2>
          <p data-aos="fade-up" data-aos-delay="100" className="text-xl text-slate-600 dark:text-slate-300 mb-10 relative z-10">
            Join the decentralized trade finance revolution on Stellar.
          </p>
          <div data-aos="fade-up" data-aos-delay="200" className="relative z-10">
            <Link 
              href="/borrow"
              className="inline-flex items-center gap-2 px-8 py-4 rounded-full bg-primary-500 text-white font-bold text-lg hover:bg-primary-600 transition-all active:scale-95 shadow-xl shadow-primary-500/30"
            >
              Start Borrowing <ArrowRight className="w-5 h-5" />
            </Link>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-slate-200 dark:border-slate-800 py-12 px-6 mt-auto">
        <div className="max-w-7xl mx-auto flex flex-col md:flex-row items-center justify-between gap-6">
          <div className="flex items-center">
            <img src="/wordmark-dark.png" alt="Aero" className="h-8 dark:hidden object-contain" />
            <img src="/wordmark-light.png" alt="Aero" className="h-8 hidden dark:block object-contain" />
          </div>
          <div className="flex gap-6 text-sm text-slate-600 dark:text-slate-400">
            <Link href="#" className="hover:text-primary-500 transition-colors">Documentation</Link>
            <Link href="#" className="hover:text-primary-500 transition-colors">Twitter</Link>
            <Link href="#" className="hover:text-primary-500 transition-colors">Discord</Link>
          </div>
          <div className="text-sm text-slate-500">
            © {new Date().getFullYear()} Aero Protocol. All rights reserved.
          </div>
        </div>
      </footer>
    </div>
  );
}
