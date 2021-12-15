// create a simple App component with two routes,
// that render two components: Dapp and Marketplace
// each route will be rendered to the DOM
import React from 'react';
import { BrowserRouter as Router, Route, Routes } from 'react-router-dom';
import {Dapp} from './Dapp';
import {Marketplace} from './Marketplace';

function App(props) {
    return (
        <div className="App">
            <Router>
                <Routes>
                    <Route path="/auction" element={<Dapp/>} exact/>
                    <Route path="/" element={<Marketplace/>} exact/>
                </Routes>
            </Router>
        </div>
    );
}

export default App;
