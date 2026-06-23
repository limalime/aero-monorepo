"use client";

import { useState } from "react";
import Link from "next/link";
import Image from "next/image"
import { usePathname } from "next/navigation";
import { Moon, Sun, Wallet, Menu, X } from "lucide-react";
import { useTheme } from "next-themes";
import { useWallet } from "@/hooks/use-wallet";

import lightLogo from "../../public/light.png";
import darkLogo from "../../public/dark.png"; 

export function Navbar() {
  const pathname = usePathname();
  const { theme, setTheme } = useTheme();
  const { address, isConnecting, connect, disconnect } = useWallet();
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

  const links = [
    { href: "/markets", label: "Markets" },
    { href: "/borrow", label: "Borrow" },
    { href: "/dashboard", label: "Dashboard" },
  ];

  const formatAddress = (addr: string) =>
    `${addr.substring(0, 5)}...${addr.substring(addr.length - 4)}`;

  const connectButton = (
    address ? (
      <button
        onClick={disconnect}
        className="px-6 py-2.5 rounded-full bg-slate-100 dark:bg-slate-800 text-slate-900 dark:text-white font-medium hover:bg-slate-200 dark:hover:bg-slate-700 transition-all active:scale-95 border border-slate-200 dark:border-slate-700 shadow-sm w-full md:w-auto"
      >
        {formatAddress(address)}
      </button>
    ) : (
      <button
        onClick={connect}
        disabled={isConnecting}
        className="px-6 py-2.5 rounded-full bg-primary-500 text-white font-medium hover:bg-primary-600 transition-all active:scale-95 shadow-lg shadow-primary-500/30 flex items-center justify-center gap-2 disabled:opacity-70 disabled:active:scale-100 w-full md:w-auto"
      >
        <Wallet className="w-4 h-4" />
        {isConnecting ? "Connecting..." : "Connect Wallet"}
      </button>
    )
  );

  return (
    <nav className="sticky top-0 z-50 glass-nav">
      <div className="max-w-7xl mx-auto px-6 h-20 flex items-center justify-between">
        <Link href="/" className="flex items-center gap-2">
          <div className="w-8 h-8 flex items-center">
            <div className="flex items-center">
              <Image
              src={lightLogo}
              width={100}
              height={80}
              alt="Aero logo"
              className="hidden dark:block"/>

            <Image
              src={darkLogo}
              width={100}
              height={80}
              alt="Aero logo"
              className="block dark:hidden"/>
            </div>
          </div>
          <span className="font-bold text-xl tracking-tight text-slate-900 dark:text-white">Aero</span>
        </Link>

        {/* Desktop Links */}
        <div className="hidden md:flex items-center gap-8">
          {links.map((link) => {
            const isActive = pathname === link.href;
            return (
              <Link
                key={link.href}
                href={link.href}
                className={`font-medium transition-colors hover:text-primary-500 ${
                  isActive ? "text-primary-500" : "text-slate-600 dark:text-slate-300"
                }`}
              >
                {link.label}
              </Link>
            );
          })}
        </div>

        {/* Desktop & Mobile Toggles */}
        <div className="flex items-center gap-2 md:gap-4">
          <button
            onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
            className="p-2 rounded-full hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors active:scale-95"
            aria-label="Toggle theme"
          >
            <Sun className="w-5 h-5 hidden dark:block text-slate-300" />
            <Moon className="w-5 h-5 block dark:hidden text-slate-600" />
          </button>

          <div className="hidden md:block">
            {connectButton}
          </div>

          <button
            onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
            className="md:hidden p-2 rounded-full hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors active:scale-95"
            aria-label="Toggle mobile menu"
          >
            {isMobileMenuOpen ? (
              <X className="w-6 h-6 text-slate-600 dark:text-slate-300" />
            ) : (
              <Menu className="w-6 h-6 text-slate-600 dark:text-slate-300" />
            )}
          </button>
        </div>
      </div>

      {/* Mobile Menu */}
      {isMobileMenuOpen && (
        <div className="md:hidden absolute top-20 left-0 right-0 bg-white dark:bg-slate-950 border-b border-slate-200 dark:border-slate-800 shadow-xl px-6 py-4 flex flex-col gap-4 animate-in slide-in-from-top-2">
          {links.map((link) => {
            const isActive = pathname === link.href;
            return (
              <Link
                key={link.href}
                href={link.href}
                onClick={() => setIsMobileMenuOpen(false)}
                className={`font-medium py-2 transition-colors hover:text-primary-500 ${
                  isActive ? "text-primary-500" : "text-slate-600 dark:text-slate-300"
                }`}
              >
                {link.label}
              </Link>
            );
          })}
          <div className="pt-4 border-t border-slate-100 dark:border-slate-800">
            {connectButton}
          </div>
        </div>
      )}
    </nav>
  );
}
