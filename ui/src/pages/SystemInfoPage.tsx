import {FC, useEffect, useMemo, useState} from 'react'
import { useTranslation } from 'react-i18next'
import {prependAPIKeyOnAuthEnabled} from '../utils/Utilities'
import { CustomGaugeChart } from '../components/CustomGaugeChart'
import { Heading1 } from '../components/Heading1'
import { Heading3 } from '../components/Heading3'
import { Loading } from '../components/Loading'
import 'material-symbols/outlined.css'
import useCommon from "../store/CommonSlice";
import {$api, client} from "../utils/http";
import {components} from "../../schema";
import {LoadingSkeletonDD} from "../components/ui/LoadingSkeletonDD";
import {ChartLoadingSkeleton} from "../components/ui/ChartLoadingSkeleton";
import {LoadingSkeletonSpan} from "../components/ui/LoadingSkeletonSpan";

type VersionInfoModel = {
    commit: string,
    version: string,
    ref: string,
    ci: string,
    time: string,
    os: string
}

export const SystemInfoPage: FC = () => {
    const configModel = useCommon(state => state.configModel)
    const user = $api.useQuery('get', '/api/v1/users/{username}', {
        params: {
            path: {
                username: 'me'
            }
        },
    })
    const systemInfo = $api.useQuery('get', '/api/v1/sys/info')
    const infoVersion = $api.useQuery('get', '/api/v1/info')

    const { t } = useTranslation()

    const gigaByte = Math.pow(10,9)
    const megaByte = Math.pow(10,6)

    const calculateFreeDiskSpace = (disk: components["schemas"]["SysExtraInfo"]["disks"]) => {
        const used = disk.reduce((x, y) => {
            return (x + (y.total_space - y.available_space))
        }, 0)

        const available = disk.reduce((x, y) => {
            return (x + y.available_space)
        }, 0)

        const total = disk.reduce((x, y) => {
            return (x + y.total_space)
        }, 0)

        return { used, available, total }
    }


    const calcedPodcastSize = useMemo(()=>{
        if (!systemInfo.data) {
            return ''
        }
        if (systemInfo.data.podcast_directory > gigaByte) {
            return (systemInfo.data.podcast_directory / gigaByte).toFixed(2) + ' GB'
        }
        else if (systemInfo.data.podcast_directory < gigaByte) {
            return (systemInfo.data.podcast_directory / megaByte).toFixed(2) + ' MB'
        }

    }, [systemInfo])


    const linkToRSSFeed = useMemo(()=>{
        if (!configModel || !user.data) {
            return ''
        }
        return prependAPIKeyOnAuthEnabled(configModel!.rssFeed, user.data)
    }, [configModel, user])


    return (
        <>
            <Heading1 className="mb-10">{t('system-info')}</Heading1>

            <div className="grid xs:grid-cols-2 lg:grid-cols-3 gap-x-8 gap-y-8 md:gap-y-16">
                {/* CPU */}
                <div className="p-8 rounded-xl shadow-[0_4px_32px_rgba(0,0,0,calc(var(--shadow-opacity)-0.1))]">
                    <span className="flex items-center gap-2 mb-2">
                        <span className="material-symbols-outlined text-(--fg-icon-color)">memory</span>
                        <Heading3>{t('cpu-usage')}</Heading3>
                    </span>
                    {
                        (systemInfo.isLoading ||!systemInfo.data) ?
                            <ChartLoadingSkeleton/>:
                    <CustomGaugeChart fill={['#10b981', '#064e3b']} labels={[t('used-cpu'), t('free-cpu')]} labelUnit="percent" max={100} value={systemInfo.data.system.cpus.global} />
                    }
                </div>

                {/* Memory */}
                <div className="p-8 rounded-xl shadow-[0_4px_32px_rgba(0,0,0,calc(var(--shadow-opacity)-0.1))]">
                    <span className="flex items-center gap-2 mb-2">
                        <span className="material-symbols-outlined text-(--fg-icon-color)">memory_alt</span>
                        <Heading3>{t('memory-usage')}</Heading3>
                    </span>
                    {
                        (systemInfo.isLoading ||!systemInfo.data) ?
                            <ChartLoadingSkeleton/>:
                            <CustomGaugeChart fill={['#c4b5fd', '#6d28d9']} labels={[t('used-memory'), t('free-memory')]} labelUnit="capacity" max={systemInfo.data.system.mem_total} value={systemInfo.data.system.mem_total - systemInfo.data.system.mem_available} />
                    }

                </div>

                {/* Disk */}
                <div className="p-8 rounded-xl shadow-[0_4px_32px_rgba(0,0,0,calc(var(--shadow-opacity)-0.1))]">
                    <span className="flex items-center gap-2 mb-2">
                        <span className="material-symbols-outlined text-(--fg-icon-color)">hard_drive</span>
                        <Heading3>{t('disk-usage')}</Heading3>
                    </span>
                    {
                        (systemInfo.isLoading ||!systemInfo.data) ?
                            <ChartLoadingSkeleton/>:
                            <CustomGaugeChart fill={['#fcd34d', '#d97706']} labels={[t('used-disk'), t('free-disk')]} labelUnit="capacity" max={calculateFreeDiskSpace(systemInfo.data.disks).total} value={calculateFreeDiskSpace(systemInfo.data.disks).used} />
                    }
                </div>

                {/* Hardware info */}
                <div>
                    <Heading3 className="mb-6">{t('hardware')}</Heading3>

                    <dl className="grid lg:grid-cols-2 gap-2 lg:gap-6 text-sm">
                        <dt className="font-medium text-(--fg-color)">{t('cpu-brand')}</dt>
                        <LoadingSkeletonDD text={systemInfo.data?.system.cpus.cpus[0]?.brand} loading={systemInfo.isLoading}/>
                        <dt className="font-medium text-(--fg-color)">{t('cpu-cores')}</dt>

                        <LoadingSkeletonDD text={systemInfo.data?.system.cpus.cpus.length} loading={systemInfo.isLoading}/>


                        <dt className="font-medium text-(--fg-color)">{t('podcast-size')}</dt>
                        <LoadingSkeletonDD loading={systemInfo.isLoading} text={calcedPodcastSize}/>
                    </dl>
                </div>

                {/* System configuration */}
                <div className="col-span-1 xs:col-span-2">
                    <Heading3 className="mb-6">{t('system-configuration')}</Heading3>

                    <dl className="grid grid-cols-1 xs:grid-cols-[auto_auto] gap-2 xs:gap-6 text-sm">
                        <dt className="font-medium text-(--fg-color)">{t('podindex-configured')}</dt>
                        <dd className="text-(--fg-secondary-color)">
                            {configModel?.podindexConfigured ? (
                                <span className="material-symbols-outlined text-(--success-fg-color)">check_circle</span>
                            ) : (
                                <span className="material-symbols-outlined text-(--danger-fg-color)">block</span>
                            )}
                        </dd>


                        <dt className="font-medium text-(--fg-color)">{t('rss-feed')}</dt>
                        <dd className="text-(--fg-secondary-color)"><a className="text-(--accent-color) hover:text-(--accent-color-hover)" href={linkToRSSFeed} target="_blank" rel="noopener noreferrer"><LoadingSkeletonSpan text={linkToRSSFeed} loading={user.isLoading}/></a></dd>
                            <>
                                <dt className="font-medium text-(--fg-color)">{t('version')}</dt>
                                <LoadingSkeletonDD text={infoVersion?.data?.version} loading={infoVersion.isLoading}></LoadingSkeletonDD>

                                <dt className="font-medium text-(--fg-color)">{t('commit')}</dt>
                                <LoadingSkeletonDD text={infoVersion.data?.commit} loading={infoVersion.isLoading}></LoadingSkeletonDD>

                                <dt className="font-medium text-(--fg-color)">{t('ci-build')}</dt>
                                <LoadingSkeletonDD text={infoVersion?.data?.ci} loading={infoVersion.isLoading}></LoadingSkeletonDD>

                                <dt className="font-medium text-(--fg-color)">{t('build-date')}</dt>
                                <LoadingSkeletonDD text={infoVersion?.data?.time} loading={infoVersion.isLoading}></LoadingSkeletonDD>
                                <dt className="font-medium text-(--fg-color)">{t('branch')}</dt>
                                <LoadingSkeletonDD text={infoVersion?.data?.ref} loading={infoVersion.isLoading}></LoadingSkeletonDD>
                                <dt className="font-medium text-(--fg-color)">{t('os')}</dt>
                                <LoadingSkeletonDD text={infoVersion?.data?.os} loading={infoVersion.isLoading}></LoadingSkeletonDD>
                            </>
                    </dl>
                </div>
            </div>
        </>
    )
}
