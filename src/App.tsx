import React, { useEffect, useState } from 'react';
import logo from './logo.svg';
import './App.css';
import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';

const App: React.FC = () => {
  const [message, setMessage] = useState("");
  const [receiveValue, setReceiveValue] = useState("");

  useEffect(() => {
    let unlisten;
    (async () => {
      unlisten = await listen('serial_receiver', event => {
        setReceiveValue(`receive: ${event.payload}`);
      })
    })();
  })

  return (
    <div className="App">
      <header className="App-header">
        <img src={logo} className="App-logo" alt="logo" />
        <p>{receiveValue}</p>
        <div style={{ flexDirection: "row" }}>
          <input
            type="text"
            style={{ fontSize: 20, paddingTop: 2, paddingBottom: 3, margin: 2, border: "none" }}
            value={message}
            onChange={event => setMessage(event.target.value)}
          />
          <button style={{ fontSize: 20, marginLeft: 20 }} onClick={() => { invoke('send_p', { message }) }}>Push It!!</button>
        </div>
      </header>
    </div>
  );
}

export default App;
