"use client";

import { useState, useEffect } from "react";
import {
  isConnected,
  setAllowed,
  requestAccess,
  getAddress,
} from "@stellar/freighter-api";
import { toast } from "sonner";

export function useWallet() {
  const [address, setAddress] = useState<string | null>(null);
  const [isConnecting, setIsConnecting] = useState(false);

  useEffect(() => {
    checkConnection();
  }, []);

  const checkConnection = async () => {
    try {
      const connectedRes = await isConnected();
      if (connectedRes.isConnected) {
        const allowedRes = await setAllowed();
        if (allowedRes.isAllowed) {
          const res = await getAddress();
          if (res.address) setAddress(res.address);
        }
      }
    } catch (e) {
      console.error(e);
    }
  };

  const connect = async () => {
    setIsConnecting(true);
    try {
      const connectedRes = await isConnected();
      if (!connectedRes.isConnected) {
        toast.error("Freighter wallet not installed", {
          description: "Please install the Freighter browser extension.",
        });
        setIsConnecting(false);
        return;
      }
      
      const res = await requestAccess();
      if (res.error) {
        toast.error(res.error);
        return;
      }
      if (res.address) {
        setAddress(res.address);
        toast.success("Wallet connected successfully!");
      }
    } catch (error) {
      console.error(error);
      toast.error("Failed to connect wallet");
    } finally {
      setIsConnecting(false);
    }
  };

  const disconnect = () => {
    setAddress(null);
    toast.info("Wallet disconnected");
  };

  return { address, isConnecting, connect, disconnect };
}
