import React, {createRef, FC, useEffect, useMemo, useState} from "react";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setCurrentTimeUpdatePercentage} from "../store/AudioPlayerSlice";
import {logCurrentPlaybackTime} from "../utils/Utilities";

type ProgressBarProps = {
    audioplayerRef: React.RefObject<HTMLAudioElement>
}

const convertToMinutes = (time: number|undefined)=>{
    if(time===undefined){
        return "00:00:00"
    }
    const timeToConvert = Number(time?.toFixed(0))
    let hours = Math.floor(timeToConvert / 3600);

    let minutes = Math.floor(timeToConvert / 60);
    let seconds = timeToConvert % 60;
    let minutes_p = String(minutes).padStart(2, "0");
    let hours_p = String(hours).padStart(2, "0");
    let seconds_p = String(seconds).padStart(2, "0");
    if(hours_p==="00"){
        return minutes_p + ":" + seconds_p.substring(0,2);
    }
    return hours_p + ":" + minutes_p + ":" + seconds_p.substring(0,2);
}

const ProgressBar:FC<ProgressBarProps> = ({audioplayerRef}) => {
    window.addEventListener("mousedown", (e) => {
        setMousePressed(true)
    })
    window.addEventListener("mouseup", (e) => {
        setMousePressed(false)
    })
    const minute = useAppSelector(state=>state.audioPlayer.metadata?.currentTime)
    const [mousePressed, setMousePressed] = useState(false);
    const metadata = useAppSelector(state=>state.audioPlayer.metadata)
    const dispatch = useAppDispatch()
    const control = createRef<HTMLElement>()
    const wrapper = createRef<HTMLDivElement>()
    const time = useAppSelector(state=>state.audioPlayer.metadata?.currentTime)
    const currentPodcastEpisode = useAppSelector(state=>state.audioPlayer.currentPodcastEpisode)

    const totalDuration = useMemo(()=>{
        console.log("Total duration: "+metadata?.duration)
        return convertToMinutes(metadata?.duration)
    },[metadata?.duration])

    const currentTime = useMemo(()=>{
        return convertToMinutes(minute)
    },[minute])

    if(audioplayerRef===undefined || audioplayerRef.current===undefined|| metadata===undefined){
        return <div>test</div>
    }

    const endWrapperPosition = (e: React.MouseEvent<HTMLDivElement>)=> {
        const offset = wrapper.current?.getBoundingClientRect()
        if (offset) {
            const localX = e.clientX - offset.left;
            const percentage = localX / offset.width * 100
            console.log("Percentage: " + percentage)
            if (percentage && audioplayerRef.current) {
                audioplayerRef.current.currentTime = Math.floor(percentage / 100 * audioplayerRef.current.duration)
                if(time && currentPodcastEpisode){
                    logCurrentPlaybackTime(currentPodcastEpisode.episode_id,Number(audioplayerRef.current.currentTime.toFixed(0)))
                }
            }
        }
    }

    const calcTotalMovement = (e: React.MouseEvent<HTMLElement, MouseEvent>)=>{
        if(mousePressed && metadata && audioplayerRef.current){
            console.log("Changing percentage to: "+(metadata.percentage + e.movementX))
            dispatch(setCurrentTimeUpdatePercentage(metadata.percentage + e.movementX))
            audioplayerRef.current.currentTime = Math.floor(metadata.percentage + e.movementX / 100 * audioplayerRef.current.duration)
        }
    }



    return (
        <div className="relative h-4 ml-5 mr-5 cursor-pointer w-11/12 box-border" id="audio-progress-wrapper" ref={wrapper} onClick={(e)=>{
            endWrapperPosition(e)

        }}>
            <div className="absolute -top-6 opacity-0 invisible timecounter" id="timecounter">{currentTime}</div>
            <div className="absolute right-0 -top-6 opacity-0 invisible timecounter">{totalDuration}</div>
            <div className="bg-gray-500 h-1" id="audio-progress" style={{width: (metadata.percentage) +"%"}}>
                <i className="fa-solid text-gray-300 fa-circle opacity-0 invisible" id="sound-control" ref={control}
                   onMouseMove={(e)=>calcTotalMovement(e)}>
                </i>
            </div>
        </div>
    );
};

export default ProgressBar;
