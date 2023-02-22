import {createRef, FC, useEffect, useMemo, useState} from "react";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setCurrentTimeUpdatePercentage} from "../store/AudioPlayerSlice";

type ProgressBarProps = {
    audioplayerRef: React.RefObject<HTMLAudioElement>
}


const ProgressBar:FC<ProgressBarProps> = ({audioplayerRef}) => {
    window.addEventListener("mousedown", (e) => {
        setMousePressed(true)
    })
    window.addEventListener("mouseup", (e) => {
        setMousePressed(false)
    })
    const maxProgress = 90;
    const minProgress = 0;
    const [mousePressed, setMousePressed] = useState(false);
    const metadata = useAppSelector(state=>state.audioPlayer.metadata)
    const dispatch = useAppDispatch()
    const control = createRef<HTMLElement>()
    const wrapper = createRef<HTMLDivElement>()

    if(audioplayerRef===undefined || audioplayerRef.current===undefined){
        return <div>test</div>
    }

    const endWrapperPosition = (e: React.MouseEvent<HTMLDivElement>)=> {
        const offset = wrapper.current?.getBoundingClientRect()
        if (offset) {
            const localX = e.clientX - offset.left;
            const percentage = localX / offset.width * 100
            if (percentage >= minProgress && percentage <= maxProgress && audioplayerRef.current) {
                audioplayerRef.current.currentTime = Math.floor(percentage / 100 * audioplayerRef.current.duration)
            }
        }
    }

    const calcTotalMovement = (e: React.MouseEvent<HTMLElement, MouseEvent>)=>{
        if(mousePressed && metadata && metadata.percentage>=minProgress && metadata.percentage<maxProgress && audioplayerRef.current){
            dispatch(setCurrentTimeUpdatePercentage(metadata.percentage + e.movementX))
            audioplayerRef.current.currentTime = Math.floor(metadata.percentage + e.movementX / 100 * audioplayerRef.current.duration)
        }
    }

    return (
        <div className="h-4 ml-5 mr-5 cursor-pointer" id="audio-progress-wrapper" ref={wrapper} onClick={(e)=>{
            endWrapperPosition(e)
        }}>
            <div className="bg-gray-500 h-1" id="audio-progress" style={{width: metadata?.percentage +"%"}}>
                <i className="fa-solid text-gray-300 fa-circle opacity-0 hidden" id="sound-control" ref={control}
                   onMouseMove={(e)=>calcTotalMovement(e)}>
                </i>
            </div>
        </div>
    );
};

export default ProgressBar;
