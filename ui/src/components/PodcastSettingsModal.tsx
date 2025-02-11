import * as Dialog from '@radix-ui/react-dialog';
import {FC, useEffect, useMemo, useState} from "react";
import {Switcher} from "./Switcher";
import useCommon, {Podcast} from "../store/CommonSlice";
import useAudioPlayer from "../store/AudioPlayerSlice";
import {PodcastSetting} from "../models/PodcastSetting";
import {CustomButtonSecondary} from "./CustomButtonSecondary";
import {useTranslation} from "react-i18next";
import {SettingsInfoIcon} from "./SettingsInfoIcon";
import {CustomInput} from "./CustomInput";
import {CustomSelect} from "./CustomSelect";
import {options} from "./SettingsNaming";
import {components} from "../../schema";
import { client } from '../utils/http';

type PodcastSettingsModalProps = {
    open: boolean,
    setOpen: (open: boolean) => void,
    podcast: components["schemas"]["PodcastDto"]
}

export const PodcastSettingsModal:FC<PodcastSettingsModalProps> = ({setOpen,open, podcast})=>{
    const setCurrentPodcast = useAudioPlayer(state => state.setCurrentPodcast)
    const updatePodcastArray = useCommon(state => state.updatePodcast)
    const [podcastSettings, setPodcastSettings] = useState<PodcastSetting>()
    const [loaded, setLoaded] = useState<boolean>(false)
    const {t} = useTranslation()
    const isSettingsEnabled = useMemo(()=>{
        return podcastSettings != null && loaded && podcastSettings.activated
    }, [podcastSettings, loaded])



    useEffect(() => {
        client.GET("/api/v1/podcasts/{id}/settings", {
            params: {
                path: {
                    id: podcast.id
                }
            }
        }).then((res)=>{
            if (res != null) {
                setPodcastSettings(res.data)
            }
            setLoaded(true)
        })
    }, []);


    useEffect(() => {
        if (loaded && !open && podcastSettings) {
            client.PUT("/api/v1/podcasts/{id}/settings", {
                body: podcastSettings,
                params: {
                    path: {
                        id: podcast.id
                    }
                }
            })
        }
    }, [loaded, open, podcastSettings]);


    return <Dialog.Root open={open}>
        <Dialog.Portal>
        <Dialog.Overlay onClick={()=>setOpen(false)} className="fixed inset-0 grid place-items-center bg-[rgba(0,0,0,0.5)] backdrop-blur-sm overflow-y-auto overflow-x-hidden z-30 transition-opacity opacity-100" />
            <Dialog.Content onClick={()=>setOpen(false)} className="fixed inset-0 grid place-items-center z-40">
                <div onClick={(e)=>e.stopPropagation()}
                    className={"relative bg-(--bg-color) max-w-2xl p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] " }>
                    <Dialog.Title className="text-(--accent-color) text-2xl">Settings</Dialog.Title>
                    <Dialog.Description className="text-(--fg-color)">Configure your podcast settings</Dialog.Description>
                    <Dialog.Close className="top-5 absolute right-5" onClick={()=>{
                        setOpen(false)
                    }}> <span
                        className="material-symbols-outlined text-(--modal-close-color) hover:text-(--modal-close-color-hover)">close</span>
                        <span className="sr-only">Close modal</span></Dialog.Close>
                    <hr className="mb-5 mt-1 border-[1px] border-(--border-color)"/>
                    <div className={`grid grid-cols-3 gap-5 ${!isSettingsEnabled && 'opacity-50'}`}>
                        <h2 className="text-(--fg-color) col-span-2">{t('episode-numbering')}</h2>
                        <Switcher className="justify-self-end" disabled={!isSettingsEnabled}
                                  checked={podcastSettings?.episodeNumbering}
                                  onChange={(checked) => {
                                      setPodcastSettings({...podcastSettings!, episodeNumbering: checked})
                                  }}/>
                        <label className="mr-6 text-(--fg-color)" htmlFor="auto-cleanup">{t('auto-cleanup')}</label>
                        <CustomButtonSecondary disabled={!isSettingsEnabled} onClick={() => {
                            client.PUT("/api/v1/settings/runcleanup")
                        }}>{t('run-cleanup')}</CustomButtonSecondary>
                        <Switcher disabled={!isSettingsEnabled}
                                  checked={podcastSettings ? podcastSettings.autoCleanup : false}
                                  className="xs:justify-self-end" id="auto-cleanup"
                                  onChange={() => {
                                      setPodcastSettings({
                                          ...podcastSettings!,
                                          autoCleanup: !podcastSettings!.autoCleanup
                                      })
                                  }}/>
                        <label htmlFor="days-to-keep"
                               className="flex gap-1 col-span-2 text-(--fg-color)">{t('days-to-keep')}<SettingsInfoIcon
                            headerKey="days-to-keep" textKey="days-to-keep-explanation"/></label>
                        <CustomInput disabled={!isSettingsEnabled} className="w-20 justify-self-end" id="days-to-keep"
                                     onChange={(e) => {
                                         setPodcastSettings({
                                             ...podcastSettings!,
                                             autoCleanupDays: parseInt(e.target.value)
                                         })
                                     }} type="number" value={podcastSettings ? podcastSettings!.autoCleanupDays : '0'}/>

                        <label htmlFor="auto-update"
                               className="flex gap-1 col-span-2 text-(--fg-color)">{t('auto-update')} <SettingsInfoIcon
                            headerKey="auto-update" textKey="auto-update-explanation"/></label>
                        <Switcher disabled={!isSettingsEnabled}
                                  checked={podcastSettings ? podcastSettings.autoUpdate : false}
                                  className="xs:justify-self-end" id="auto-update"
                                  onChange={() => {
                                      setPodcastSettings({
                                          ...podcastSettings!,
                                          autoUpdate: !podcastSettings?.autoUpdate
                                      })
                                  }}/>

                        <label htmlFor="auto-download"
                               className="flex gap-1 col-span-2 text-(--fg-color)">{t('auto-download')}
                            <SettingsInfoIcon
                                headerKey="auto-download" textKey="auto-download-explanation"/></label>
                        <Switcher disabled={!isSettingsEnabled}
                                  checked={podcastSettings ? podcastSettings.autoDownload : false}
                                  className="xs:justify-self-end" id="auto-download"
                                  onChange={() => {
                                      setPodcastSettings({
                                          ...podcastSettings!,
                                          autoDownload: !podcastSettings?.autoDownload
                                      })
                                  }}/>
                        <label className="text-(--fg-color) flex gap-1 col-span-2"
                               htmlFor="colon-replacement">{t('colon-replacement')}
                            <SettingsInfoIcon className="mb-auto" headerKey="colon-replacement"
                                              textKey="colon-replacement-explanation"/>
                        </label>
                        <CustomSelect disabled={!isSettingsEnabled} id="colon-replacement" options={options}
                                      onChange={(v) => {
                                          setPodcastSettings({
                                              ...podcastSettings!,
                                              replacementStrategy: v
                                          })
                                      }}
                                      value={podcastSettings ? podcastSettings!.replacementStrategy : options[0]!.value}/>
                        <label htmlFor="number-of-podcasts-to-download"
                               className="flex gap-1 col-span-2 text-(--fg-color)">{t('number-of-podcasts-to-download')}
                            <SettingsInfoIcon
                                headerKey="number-of-podcasts-to-download"
                                textKey="number-of-podcasts-to-download-explanation"/></label>
                        <CustomInput disabled={!isSettingsEnabled} className="w-20 justify-self-end" id="number-of-podcasts-to-download" onChange={(e) => {
                            setPodcastSettings({...podcastSettings!, podcastPrefill: parseInt(e.target.value)})
                        }} type="number" value={podcastSettings ? podcastSettings.podcastPrefill : 5}/>
                        <label htmlFor="activate"
                               className="flex gap-1 col-span-2 text-(--fg-color)">{t('activated')}
                            <SettingsInfoIcon
                                headerKey="auto-download" textKey="auto-download-explanation"/></label>
                        <Switcher
                            checked={podcastSettings ? podcastSettings.activated : false}
                            className="xs:justify-self-end" id="activate"
                            onChange={() => {
                                if (!podcastSettings && loaded) {
                                    setPodcastSettings({
                                        podcastId: podcast.id,
                                        episodeNumbering: false,
                                        autoDownload: false,
                                        autoUpdate: false,
                                        autoCleanup: false,
                                        autoCleanupDays: 0,
                                        replaceInvalidCharacters: false,
                                        useExistingFilename: false,
                                        replacementStrategy: options[0]!.value,
                                        episodeFormat: "{}",
                                        podcastFormat: "{}",
                                        directPaths: false,
                                        activated: true,
                                        podcastPrefill: 5,
                                    })
                                } else {
                                    setPodcastSettings({
                                        ...podcastSettings!,
                                        activated: !podcastSettings?.activated
                                    })
                                }

                            }}/>
                    </div>
                    {!isSettingsEnabled &&
                        <div className="col-span-3 text-red-500">Settings are not enabled for this podcast</div>}
                </div>
            </Dialog.Content>
        </Dialog.Portal>
    </Dialog.Root>
}
