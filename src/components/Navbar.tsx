import React from 'react';

interface NavbarProps {
  environment: 'web' | 'tauri';
  isConnected?: boolean;
}

const Navbar: React.FC<NavbarProps> = ({ environment, isConnected = false }) => {
  return (
    <nav className="navbar">
      <div className="navbar-content">
        <a href="/" className="navbar-brand">
          <div className="logo">M</div>
          <span>Miden VM Playground</span>
        </a>
        
        <ul className="navbar-nav">
          <li>
            <a href="https://github.com/dhananjaypai08/masm-code-playground">Github</a>
          </li>
          <li>
            <a href="https://0xmiden.github.io/miden-vm/user_docs/assembly/main.html" target="_blank" rel="noopener noreferrer">
              Documentation
            </a>
          </li>
          <li>
            <a href="https://docs.rs/miden-vm/latest/miden_vm/" target="_blank" rel="noopener noreferrer">
              Miden Assembly SDK
            </a>
          </li>
          <li>
            <a href="https://github.com/0xMiden/miden-vm" target="_blank" rel="noopener noreferrer">
              Miden-VM GitHub
            </a>
          </li>
        </ul>
        
        <div className="navbar-status">
          <span className={`status-indicator ${environment === 'web' ? (isConnected ? 'connected' : 'disconnected') : 'connected'}`}>
            {environment === 'tauri' ? (
              <>
                <span>🖥️</span>
                <span>Desktop App</span>
              </>
            ) : (
              <>
                <span>{isConnected ? '🟢' : '🔴'}</span>
                <span>Web {isConnected ? 'Connected' : 'Disconnected'}</span>
              </>
            )}
          </span>
        </div>
      </div>
    </nav>
  );
};

export default Navbar;