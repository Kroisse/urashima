import React, { useState } from 'react';

import MainPage from './components/MainPage';

const App: React.FC = () => {
    const [value, setValue] = useState('');
    return <MainPage value={value} onChange={v => setValue(v)} />;
};

export default App;
