import React, { useEffect, useRef } from 'react';

import { NaruRuntime, Capsule as NaruCapsule } from '../naru';

export interface CapsuleProps {
    code: string;
}

export default function Capsule({ code = '' }: CapsuleProps) {
    const capsule = useRef<NaruCapsule>();
    useEffect(() => {
        const rt = new NaruRuntime();
        const cap = rt.capsule();
        capsule.current = cap;
        return () => {
            cap.free();
            rt.free();
        };
    }, []);
    useEffect(() => {
        if (capsule.current) {
            try {
                capsule.current.eval(code);
            } catch (err) {
                console.error(err);
            }
        }
    }, [code]);
    return <div></div>;
}
