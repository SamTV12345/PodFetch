import * as Dialog from '@radix-ui/react-dialog';
import {FC, useMemo} from "react";
import {Switcher} from "./Switcher";
import {PodcastSetting} from "../models/PodcastSetting";
import {CustomButtonSecondary} from "./CustomButtonSecondary";
import {useTranslation} from "react-i18next";
import {SettingsInfoIcon} from "./SettingsInfoIcon";
import {CustomInput} from "./CustomInput";
import {CustomSelect} from "./CustomSelect";
import {options} from "./SettingsNaming";
import {components} from "../../schema";
import {$api, client} from '../utils/http';
import {useQueryClient} from "@tanstack/react-query";

type PodcastSettingsModalProps = {
    podcast: components["schemas"]["PodcastDto"]
}

export const PodcastSettingsModal:FC<PodcastSettingsModalProps> = ({podcast})=>{
    const {t} = useTranslation()
    const queryClient = useQueryClient()

    const podcastSettings = $api.useQuery('get', '/api/v1/podcasts/{id}/settings', {
        params: {
            path: {
                id: podcast.id
            }
        },
        retry: false
    })

    const updatePodcastSettings = $api.useMutation('put', '/api/v1/podcasts/{id}/settings')
    const isSettingsEnabled = useMemo(()=>{
        if (!podcastSettings.data) {
            return false
        }

        return podcastSettings.data.activated
    }, [podcastSettings])

    return <Dialog.Root onOpenChange={(isOpen)=>{
        if (!isOpen) {
            console.log("On close")
            updatePodcastSettings.mutate({
                body: podcastSettings.data!,
                params: {
                    path: {
                        id: podcast.id
                    }
                }
            })
        }
    }}>
        <Dialog.Trigger asChild>
            <button className="material-symbols-outlined inline cursor-pointer align-middle text-(--fg-icon-color) hover:text-(--fg-icon-color-hover)">settings</button>
        </Dialog.Trigger>
        <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 grid place-items-center bg-[rgba(0,0,0,0.5)] backdrop-blur-sm
            overflow-y-auto overflow-x-hidden z-30 transition-opacity opacity-100" />
            <Dialog.Content className="fixed inset-0 z-40 flex items-center justify-center p-4">
                <div onClick={(e)=>e.stopPropagation()}
                    className={"relative bg-(--bg-color) max-w-2xl p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] " }>
                    <Dialog.Title className="text-(--accent-color) text-2xl">{t('settings')}</Dialog.Title>
                    <Dialog.Description className="text-(--fg-color)">{t('settings-configure')}</Dialog.Description>
                    <Dialog.Close className="top-5 absolute right-5"> <span
                        className="material-symbols-outlined text-(--modal-close-color) hover:text-(--modal-close-color-hover)">close</span>
                        <span className="sr-only">Close modal</span></Dialog.Close>
                    <hr className="mb-5 mt-1 border-[1px] border-(--border-color)"/>
                    <div className={`grid grid-cols-3 gap-5 ${!isSettingsEnabled && 'opacity-50'}`}>
                        <label className="mr-6 text-(--fg-color) col-span-2" htmlFor="auto-cleanup">{t('podcast-name')}</label>
                        <CustomInput value={podcast.name} onChange={(event)=>{
                            event.target.value
                        }}/>

                        <h2 className="text-(--fg-color) col-span-2">{t('episode-numbering')}</h2>
                        <Switcher className="justify-self-end" disabled={!isSettingsEnabled}
                                  checked={podcastSettings?.data?.episodeNumbering}
                                  onChange={(checked) => {
                                      queryClient.setQueryData(['get', '/api/v1/podcasts/{id}/settings', {
                                            params: { path: { id: podcast.id } }
                                      }], (oldData: PodcastSetting | undefined) => {
                                            return {...oldData!, episodeNumbering: checked}
                                      })
                                  }}/>
                        <label className="mr-6 text-(--fg-color)" htmlFor="auto-cleanup">{t('auto-cleanup')}</label>
                        <CustomButtonSecondary disabled={!isSettingsEnabled} onClick={() => {
                            client.PUT("/api/v1/settings/runcleanup")
                        }}>{t('run-cleanup')}</CustomButtonSecondary>
                        <Switcher disabled={!isSettingsEnabled}
                                  checked={podcastSettings.data ? podcastSettings.data.autoCleanup : false}
                                  className="xs:justify-self-end" id="auto-cleanup"
                                  onChange={() => {
                                      queryClient.setQueryData(['get', '/api/v1/podcasts/{id}/settings', {
                                          params: { path: { id: podcast.id } }
                                      }], (oldData: PodcastSetting | undefined) => {
                                          return {...oldData, autoCleanup: !oldData?.autoCleanup}
                                      })
                                  }}/>
                        <label htmlFor="days-to-keep"
                               className="flex gap-1 col-span-2 text-(--fg-color)">{t('days-to-keep')}<SettingsInfoIcon
                            headerKey="days-to-keep" textKey="days-to-keep-explanation"/></label>
                        <CustomInput disabled={!isSettingsEnabled} className="w-20 justify-self-end" id="days-to-keep"
                                     onChange={(e) => {
                                         queryClient.setQueryData(['get', '/api/v1/podcasts/{id}/settings', {
                                             params: { path: { id: podcast.id } }
                                         }], (oldData: PodcastSetting | undefined) => {
                                             return {...oldData, autoCleanupDays: parseInt(e.target.value)}
                                         })
                                     }} type="number" value={podcastSettings.data ? podcastSettings.data?.autoCleanupDays : '0'}/>

                        <label htmlFor="auto-update"
                               className="flex gap-1 col-span-2 text-(--fg-color)">{t('auto-update')} <SettingsInfoIcon
                            headerKey="auto-update" textKey="auto-update-explanation"/></label>
                        <Switcher disabled={!isSettingsEnabled}
                                  checked={podcastSettings.data ? podcastSettings.data.autoUpdate : false}
                                  className="xs:justify-self-end" id="auto-update"
                                  onChange={() => {
                                      queryClient.setQueryData(['get', '/api/v1/podcasts/{id}/settings', {
                                          params: { path: { id: podcast.id } }
                                      }], (oldData: PodcastSetting | undefined) => {
                                          return {...oldData, autoUpdate: !podcastSettings?.data?.autoUpdate}
                                      })
                                  }}/>

                        <label htmlFor="auto-download"
                               className="flex gap-1 col-span-2 text-(--fg-color)">{t('auto-download')}
                            <SettingsInfoIcon
                                headerKey="auto-download" textKey="auto-download-explanation"/></label>
                        <Switcher disabled={!isSettingsEnabled}
                                  checked={podcastSettings.data ? podcastSettings.data.autoDownload : false}
                                  className="xs:justify-self-end" id="auto-download"
                                  onChange={() => {
                                      queryClient.setQueryData(['get', '/api/v1/podcasts/{id}/settings', {
                                          params: { path: { id: podcast.id } }
                                      }], (oldData: PodcastSetting | undefined) => {
                                          return {...oldData, autoDownload: !podcastSettings?.data?.autoDownload}
                                      })
                                  }}/>
                        <label className="text-(--fg-color) flex gap-1 col-span-2"
                               htmlFor="colon-replacement">{t('colon-replacement')}
                            <SettingsInfoIcon className="mb-auto" headerKey="colon-replacement"
                                              textKey="colon-replacement-explanation"/>
                        </label>
                        <CustomSelect disabled={!isSettingsEnabled} id="colon-replacement" options={options}
                                      onChange={(v) => {
                                          queryClient.setQueryData(['get', '/api/v1/podcasts/{id}/settings', {
                                              params: { path: { id: podcast.id } }
                                          }], (oldData: PodcastSetting | undefined) => {
                                              return {...oldData, replacementStrategy: v}
                                          })
                                      }}
                                      value={podcastSettings.data ? podcastSettings.data.replacementStrategy : options[0]!.value}/>
                        <label htmlFor="number-of-podcasts-to-download"
                               className="flex gap-1 col-span-2 text-(--fg-color)">{t('number-of-podcasts-to-download')}
                            <SettingsInfoIcon
                                headerKey="number-of-podcasts-to-download"
                                textKey="number-of-podcasts-to-download-explanation"/></label>
                        <CustomInput disabled={!isSettingsEnabled} className="w-20 justify-self-end" id="number-of-podcasts-to-download"
                                     onChange={(e) => {
                                         queryClient.setQueryData(['get', '/api/v1/podcasts/{id}/settings', {
                                             params: { path: { id: podcast.id } }
                                         }], (oldData: PodcastSetting | undefined) => {
                                             return {...oldData, podcastPrefill: parseInt(e.target.value)}
                                         })
                        }} type="number" value={podcastSettings.data ? podcastSettings.data.podcastPrefill : 5}/>
                        <label htmlFor="activate"
                               className="flex gap-1 col-span-2 text-(--fg-color)">{t('activated')}
                            <SettingsInfoIcon
                                headerKey="auto-download" textKey="auto-download-explanation"/></label>
                        <Switcher
                            checked={podcastSettings.data ? podcastSettings.data.activated : false}
                            className="xs:justify-self-end" id="activate"
                            onChange={() => {
                                if (!podcastSettings.data && podcastSettings.isFetched) {
                                    queryClient.setQueryData(['get', '/api/v1/podcasts/{id}/settings', {
                                            params: { path: { id: podcast.id } }
                                        }], ()=>({
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
                                    }))
                                } else {
                                    queryClient.setQueryData(['get', '/api/v1/podcasts/{id}/settings', {
                                        params: { path: { id: podcast.id } }
                                    }], (oldData: components["schemas"]["PodcastSetting"])=>{
                                        return {
                                            ...oldData!,
                                            activated: !oldData?.activated
                                        }})
                                }

                            }}/>
                    </div>
                    {!isSettingsEnabled &&
                        <div className="col-span-3 text-red-500">{t('settings-not-enabled')}</div>}
                </div>
            </Dialog.Content>
        </Dialog.Portal>
    </Dialog.Root>
}
