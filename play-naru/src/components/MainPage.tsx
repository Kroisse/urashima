import React, { useRef, useEffect, Suspense, useState } from 'react';
import MonacoEditor, { EditorDidMount } from 'react-monaco-editor';
import { KeyCode, KeyMod, editor } from 'monaco-editor';

import './MainPage.css';

const Capsule = React.lazy(() => import('./Capsule'));

const EDITOR_OPTIONS: editor.IEditorConstructionOptions = {
    fontSize: 14,
};

export interface MainPageProps {
    value: string;
    onChange: (value: string) => void;
}

export default function MainPage({ value, onChange }: MainPageProps) {
    const [code, setCode] = useState('');
    const monaco = useRef<MonacoEditor>(null);
    useEffect(() => {
        const relayout = () => {
            if (monaco.current != null && monaco.current.editor != null) {
                monaco.current.editor.layout();
            }
        };
        window.addEventListener('resize', relayout);
        return () => {
            window.removeEventListener('resize', relayout);
        };
    }, []);
    const editorDidMount: EditorDidMount = (editor, monaco) => {
        editor.addAction({
            id: 'naru-execute',
            label: 'Execute',
            keybindings: [KeyMod.CtrlCmd | KeyCode.Enter],
            run(editor) {
                setCode(editor.getValue());
            },
        });
    };
    return (
        <div className="main-page">
            <header className="header">Play Naru</header>
            <section>
                <MonacoEditor
                    ref={monaco}
                    value={value}
                    onChange={onChange}
                    theme="vs-dark"
                    editorDidMount={editorDidMount}
                    language="lua"
                    options={EDITOR_OPTIONS}
                />
            </section>
            <section>
                <Suspense fallback={<p>Loading...</p>}>
                    <Capsule code={code} />
                </Suspense>
            </section>
        </div>
    );
}
