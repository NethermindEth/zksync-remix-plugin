import { useEffect, useContext } from 'react';
import { RemixClientContext } from '../../contexts/RemixClientContext';
import { ConnectionContext } from '../../contexts/ConnectionContext';
function EthereumEventsComponent() {
    const remixClient = useContext(RemixClientContext);


    remixClient.solidity.on('providerChanged', (provider: NetworkProvider) => {
        console.log('providerChanged', provider);
      })
      
  useEffect(() => {
    if (!window.ethereum) {
      console.warn("Ethereum provider not detected");
      return;
    }

    const handleAccountsChanged = (accounts) => {
      console.log('New accounts:', accounts);
    };

    const handleChainChanged = (chainId) => {
      console.log('New chain ID:', chainId);
    };

    const handleConnect = (info) => {
      console.log('Connected:', info);
    };

    const handleDisconnect = (error) => {
      console.error('Disconnected:', error);
    };

    const handleMessage = (message) => {
      console.log('Received message:', message);
    };

    window.ethereum.on('accountsChanged', handleAccountsChanged);
    window.ethereum.on('chainChanged', handleChainChanged);
    window.ethereum.on('connect', handleConnect);
    window.ethereum.on('disconnect', handleDisconnect);
    window.ethereum.on('message', handleMessage);

    // Cleanup listeners when the component is unmounted
    return () => {
      window.ethereum.removeListener('accountsChanged', handleAccountsChanged);
      window.ethereum.removeListener('chainChanged', handleChainChanged);
      window.ethereum.removeListener('connect', handleConnect);
      window.ethereum.removeListener('disconnect', handleDisconnect);
      window.ethereum.removeListener('message', handleMessage);
    };

  }, []); // Empty dependency array ensures this useEffect runs once when the component mounts

  return (
    <div>
      {/* Your component's content here */}
    </div>
  );
}

export default EthereumEventsComponent;