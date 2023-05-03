import {createPortal} from "react-dom";
import {useState} from "react";
import {useCtrlPressed, useKeyDown} from "../hooks/useKeyDown";
import {SearchComponent} from "./SearchComponent";

export const Search = () => {
    const [open, setOpen] = useState<boolean>(false)

    useCtrlPressed(()=>{
         setOpen(!open)
         document.getElementById('search-input')!.focus()
    }, ["f"])

    useKeyDown(()=>{
        setOpen(false)
    },['Escape'])


    return createPortal(
        <div id="defaultModal" tabIndex={-1} aria-hidden="true" onClick={()=>setOpen(false)}
             className={`overflow-y-auto overflow-x-hidden fixed top-0 right-0 left-0 z-50 md:inset-0 h-modal md:h-full
             ${!open&&'pointer-events-none'}
              z-40 ${open?'opacity-100':'opacity-0'}`}>
            <div className="grid place-items-center h-screen ">
                <div className={`bg-gray-800 max-w-7xl ${open?'opacity-100':'opacity-0'}`} onClick={e=>e.stopPropagation()}>
                <SearchComponent/>
            </div>
            </div>
        </div>, document.getElementById('modal')!)
}
