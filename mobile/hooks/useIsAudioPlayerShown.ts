import {useSegments} from "expo-router";
import {useMemo} from "react";

export function useIsAudioPlayerShown() {
    const segments = useSegments();

    return useMemo(()=>{
        if (segments[0] === 'server-setup') return false;

        if (segments[0] === 'player') return false;


        return segments[0] != '(tabs)'
    }, [segments])
}