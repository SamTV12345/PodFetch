import * as Dialog from '@radix-ui/react-dialog';
import {FC, useEffect} from "react";
import {Switcher} from "./Switcher";
import useCommon, {Podcast} from "../store/CommonSlice";
import useAudioPlayer from "../store/AudioPlayerSlice";
import axios from "axios";
import {PodcastSettingsUpdate} from "../models/PodcastSettingsUpdate";

type PodcastSettingsModalProps = {
    open: boolean,
    setOpen: (open: boolean) => void,
    podcast: Podcast
}

export const PodcastSettingsModal:FC<PodcastSettingsModalProps> = ({setOpen,open, podcast})=>{
    const setCurrentPodcast = useAudioPlayer(state => state.setCurrentPodcast)
    const updatePodcastArray = useCommon(state => state.updatePodcast)


    useEffect(() => {
        axios.put("/podcasts/"+podcast.id+"/settings", {
            podcastId: podcast.id,
            episodeNumbering: podcast.episode_numbering
        } satisfies PodcastSettingsUpdate)
    }, [open]);


    return <Dialog.Root open={open}>
        <Dialog.Portal>
        <Dialog.Overlay onClick={()=>setOpen(false)} className="fixed inset-0 grid place-items-center bg-[rgba(0,0,0,0.5)] backdrop-blur overflow-y-auto overflow-x-hidden z-30 transition-opacity opacity-100" />
            <Dialog.Content onClick={()=>setOpen(false)} className="fixed inset-0 grid place-items-center z-40">
                <div onClick={(e)=>e.stopPropagation()}
                    className="relative bg-[--bg-color] max-w-2xl p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))]">
                    <Dialog.Title className="text-[--accent-color] text-2xl">Settings</Dialog.Title>
                    <Dialog.Description className="text-[--fg-color]">Configure your podcast settings</Dialog.Description>
                    <Dialog.Close className="top-5 absolute right-5" onClick={()=>{
                        setOpen(false)
                    }}> <span
                        className="material-symbols-outlined text-[--modal-close-color] hover:text-[--modal-close-color-hover]">close</span>
                        <span className="sr-only">Close modal</span></Dialog.Close>
                    <hr className="mb-5 mt-1 border-[1px] border-[--border-color]"/>
                    <div className="grid grid-cols-2 gap-5">
                        <h2 className="text-[--fg-color]">Episode Numbering</h2>
                        <Switcher checked={podcast.episode_numbering} setChecked={(checked)=>{
                            setCurrentPodcast({...podcast, episode_numbering:checked})
                            updatePodcastArray({...podcast, episode_numbering:checked})
                        }}/>
                    </div>
                </div>


            </Dialog.Content>
        </Dialog.Portal>
    </Dialog.Root>
}
