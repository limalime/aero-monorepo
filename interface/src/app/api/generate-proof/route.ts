import { NextResponse } from 'next/server';

export async function POST(req: Request) {
  try {
    const body = await req.json();
    const { amount, invoiceId, dueDate } = body;

    if (!amount || !invoiceId) {
      return NextResponse.json(
        { error: 'Missing required parameters' },
        { status: 400 }
      );
    }

    // This is the Prover Relayer endpoint.
    // In production, we would use noir_js here to generate a real ZK proof from the .acir artifact.
    // e.g.:
    // const backend = new BarretenbergBackend(circuit);
    // const noir = new Noir(circuit, backend);
    // const proof = await noir.generateFinalProof(inputs);
    
    // For now, since we don't bundle the heavy WASM to the client, we mock the 256-byte payload
    // that the Soroban contract expects as public inputs (8 fields x 32 bytes).

    const proofLength = 1024;
    const proofBytes = new Uint8Array(proofLength);
    
    // Fill proof with mock data
    for (let i = 0; i < proofLength - 256; i++) {
      proofBytes[i] = Math.floor(Math.random() * 256);
    }

    const publicInputsOffset = proofLength - 256;

    // The Rust verifier reads fields as 32-byte chunks.
    // Numbers are extracted by reading the first N bytes in little-endian.

    const writeLittleEndian = (val: number, offset: number, byteLength: number) => {
      // Create a BigInt to handle large numbers (like loan amounts)
      let bigVal = BigInt(val);
      for (let i = 0; i < byteLength; i++) {
        proofBytes[offset + i] = Number(bigVal & 0xffn);
        bigVal = bigVal >> 8n;
      }
    };

    // Field 0: invoice_hash
    // We can leave this as zeros or random for mock, it's just a BytesN<32>

    // Field 1: loan_amount (u64 little-endian)
    const loanAmountNumber = parseFloat(amount);
    writeLittleEndian(loanAmountNumber, publicInputsOffset + 32, 8);
    
    // Field 2: provider_response_hash
    
    // Field 3: nullifier
    // We need a unique nullifier so it doesn't fail the double-spend check on multiple clicks.
    writeLittleEndian(Math.floor(Date.now() / 1000), publicInputsOffset + 96, 8);
    
    // Field 4: ltv_bps (u32 little-endian, e.g. 8000 = 80%)
    writeLittleEndian(8000, publicInputsOffset + 128, 4);
    
    // Field 5: interest_bps (u32 little-endian, e.g. 500 = 5%)
    writeLittleEndian(500, publicInputsOffset + 160, 4);
    
    // Fields 6, 7 are reserved
    // Convert to hex string to send over JSON safely
    const proofHex = Buffer.from(proofBytes).toString('hex');

    return NextResponse.json({ success: true, proof: proofHex });
  } catch (error) {
    console.error('Prover Relayer Error:', error);
    return NextResponse.json(
      { error: 'Failed to generate proof' },
      { status: 500 }
    );
  }
}
