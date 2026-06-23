import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Navbar } from '../src/components/navbar';

// Mock dependencies
jest.mock('next/navigation', () => ({
  usePathname: () => '/',
}));

jest.mock('next-themes', () => ({
  useTheme: () => ({ theme: 'dark', setTheme: jest.fn() }),
}));

jest.mock('../src/hooks/use-wallet', () => ({
  useWallet: () => ({
    address: null,
    isConnecting: false,
    connect: jest.fn(),
    disconnect: jest.fn(),
  }),
}));

describe('Navbar', () => {
  it('renders the logo and links', () => {
    render(<Navbar />);
    expect(screen.getByText('Aero')).toBeInTheDocument();
    expect(screen.getByText('Markets')).toBeInTheDocument();
    expect(screen.getByText('Borrow')).toBeInTheDocument();
    expect(screen.getByText('Dashboard')).toBeInTheDocument();
  });

  it('renders the connect wallet button', () => {
    render(<Navbar />);
    expect(screen.getByText('Connect Wallet')).toBeInTheDocument();
  });
});
