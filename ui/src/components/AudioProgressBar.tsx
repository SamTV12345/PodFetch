import {createRef, useEffect, useMemo, useState} from "react";

const ProgressBar = () => {
    window.addEventListener("mousedown", (e) => {
        setMousePressed(true)
    })
    window.addEventListener("mouseup", (e) => {
        setMousePressed(false)
    })
    const maxProgress = 90;
    const minProgress = 0;
    const [progressIconPressed, setProgressIconPressed] = useState(false);
    const [mousePressed, setMousePressed] = useState(false);
    const [progressIconPosition, setProgressIconPosition] = useState(0);
    const control = createRef<HTMLElement>()
    const wrapper = createRef<HTMLDivElement>()

    const endWrapperPosition = (e: React.MouseEvent<HTMLDivElement>)=> {
        const offset = wrapper.current?.getBoundingClientRect()
        if (offset) {
            const localX = e.clientX - offset.left;
            const percentage = localX / offset.width * 100
            if (percentage >= minProgress && percentage <= maxProgress) {
                setProgressIconPosition(percentage)
            }
        }
    }

    useEffect(()=>{
        console.log("Button ist gedrückt: " + progressIconPressed)
    },[progressIconPressed])

    const calcTotalMovement = (e: React.MouseEvent<HTMLElement, MouseEvent>)=>{
        if(mousePressed && progressIconPosition>=minProgress && progressIconPosition<maxProgress){
            setProgressIconPosition(progressIconPosition + e.movementX)
        }
    }

    return (
        <div className="h-4 ml-5 mr-5" id="audio-progress-wrapper" ref={wrapper} onClick={(e)=>{
            endWrapperPosition(e)
        }}>
            <div className="bg-gray-500 h-1" id="audio-progress" style={{width:progressIconPosition +"%"}}>
                <i className="fa-solid text-gray-300 fa-circle opacity-0 hidden" id="sound-control" ref={control}
                   onMouseMove={(e)=>calcTotalMovement(e)}>
                </i>
            </div>
        </div>
    );
};

export default ProgressBar;
