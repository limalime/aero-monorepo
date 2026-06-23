import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import { ThemeProvider } from "@/components/theme-provider";
import { Navbar } from "@/components/navbar";
import { AosInit } from "@/components/aos-init";
import { Toaster } from "sonner";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "Aero | ZK Trade Finance & Invoice Factoring",
  description: "Borrow against unpaid real-world invoices while keeping business data 100% private.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={`${inter.className} min-h-screen flex flex-col relative`}>
        {/* Ambient background glows */}
        <div className="fixed inset-0 overflow-hidden pointer-events-none -z-10">
          <div className="absolute -top-1/2 -right-1/2 w-[100%] h-[100%] rounded-full bg-primary-500/10 blur-[120px] dark:bg-primary-500/5 mix-blend-screen" />
          <div className="absolute -bottom-1/2 -left-1/2 w-[100%] h-[100%] rounded-full bg-primary-400/10 blur-[120px] dark:bg-primary-900/20 mix-blend-screen" />
        </div>
        
        <ThemeProvider
          attribute="class"
          defaultTheme="dark"
          enableSystem
          disableTransitionOnChange
        >
          <AosInit />
          <Navbar />
          <main className="flex-1 flex flex-col">
            {children}
          </main>
          <Toaster
            theme="system"
            position="top-center"
            richColors/>
        </ThemeProvider>
      </body>
    </html>
  );
}
