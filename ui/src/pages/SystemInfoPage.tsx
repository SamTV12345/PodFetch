import { FC, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import axios, { AxiosResponse } from 'axios'
import {apiURL, prependAPIKeyOnAuthEnabled} from '../utils/Utilities'
import { DiskModel } from '../models/DiskModel'
import { SysExtraInfo } from '../models/SysExtraInfo'
import { CustomGaugeChart } from '../components/CustomGaugeChart'
import { Heading1 } from '../components/Heading1'
import { Heading3 } from '../components/Heading3'
import { Loading } from '../components/Loading'
import 'material-symbols/outlined.css'
import useCommon from "../store/CommonSlice";

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
    const [systemInfo, setSystemInfo] = useState<SysExtraInfo>()
    const [versionInfo, setVersionInfo] = useState<VersionInfoModel>()
    const { t } = useTranslation()

    const gigaByte = Math.pow(10,9)
    const megaByte = Math.pow(10,6)

    useEffect(() => {
        axios.get(apiURL + '/sys/info')
            .then((response: AxiosResponse<SysExtraInfo>) => setSystemInfo(response.data))
        axios.get(apiURL + '/info')
            .then(c => setVersionInfo(c.data))

        const updateInterval = setInterval(() => {
            axios.get(apiURL + '/sys/info')
                .then((response: AxiosResponse<SysExtraInfo>) => setSystemInfo(response.data))
        }, 5000)
        return () => clearInterval(updateInterval)
    }, [])

    if (!systemInfo) {
        return <Loading />
    }

    const calculateFreeDiskSpace = (disk: DiskModel[]) => {
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

    const calcPodcastSize = () => {
        if (systemInfo.podcast_directory > gigaByte) {
            return (systemInfo.podcast_directory / gigaByte).toFixed(2) + ' GB'
        }
        else if (systemInfo.podcast_directory < gigaByte) {
            return (systemInfo.podcast_directory / megaByte).toFixed(2) + ' MB'
        }
    }

    return (
        <>
            <Heading1 className="mb-10">{t('system-info')}</Heading1>

            <div className="grid xs:grid-cols-2 lg:grid-cols-3 gap-x-8 gap-y-8 md:gap-y-16">
                {/* CPU */}
                <div className="p-8 rounded-xl shadow-[0_4px_32px_rgba(0,0,0,calc(var(--shadow-opacity)-0.1))]">
                    <span className="flex items-center gap-2 mb-2">
                        <span className="material-symbols-outlined text-[--fg-icon-color]">memory</span>
                        <Heading3>{t('cpu-usage')}</Heading3>
                    </span>

                    <CustomGaugeChart fill={['#10b981', '#064e3b']} labels={[t('used-cpu'), t('free-cpu')]} labelUnit="percent" max={100} value={systemInfo.system.global_cpu_info.cpu_usage} />
                </div>

                {/* Memory */}
                <div className="p-8 rounded-xl shadow-[0_4px_32px_rgba(0,0,0,calc(var(--shadow-opacity)-0.1))]">
                    <span className="flex items-center gap-2 mb-2">
                        <span className="material-symbols-outlined text-[--fg-icon-color]">memory_alt</span>
                        <Heading3>{t('memory-usage')}</Heading3>
                    </span>

                    <CustomGaugeChart fill={['#c4b5fd', '#6d28d9']} labels={[t('used-memory'), t('free-memory')]} labelUnit="capacity" max={systemInfo.system.total_memory} value={systemInfo.system.total_memory - systemInfo.system.free_memory} />
                </div>

                {/* Disk */}
                <div className="p-8 rounded-xl shadow-[0_4px_32px_rgba(0,0,0,calc(var(--shadow-opacity)-0.1))]">
                    <span className="flex items-center gap-2 mb-2">
                        <span className="material-symbols-outlined text-[--fg-icon-color]">hard_drive</span>
                        <Heading3>{t('disk-usage')}</Heading3>
                    </span>

                    <CustomGaugeChart fill={['#fcd34d', '#d97706']} labels={[t('used-disk'), t('free-disk')]} labelUnit="capacity" max={calculateFreeDiskSpace(systemInfo.system.disks).total} value={calculateFreeDiskSpace(systemInfo.system.disks).used} />
                </div>

                {/* Hardware info */}
                <div>
                    <Heading3 className="mb-6">{t('hardware')}</Heading3>

                    <dl className="grid lg:grid-cols-2 gap-2 lg:gap-6 text-sm">
                        <dt className="font-medium text-[--fg-color]">{t('cpu-brand')}</dt>
                        <dd className="text-[--fg-secondary-color]">{systemInfo.system.global_cpu_info.brand}</dd>

                        <dt className="font-medium text-[--fg-color]">{t('cpu-cores')}</dt>
                        <dd className="text-[--fg-secondary-color]">{systemInfo.system.cpus.length}</dd>

                        <dt className="font-medium text-[--fg-color]">{t('podcast-size')}</dt>
                        <dd className="text-[--fg-secondary-color]">{calcPodcastSize()}</dd>
                    </dl>
                </div>

                {/* System configuration */}
                <div className="col-span-1 xs:col-span-2">
                    <Heading3 className="mb-6">{t('system-configuration')}</Heading3>

                    <dl className="grid grid-cols-1 xs:grid-cols-[auto_auto] gap-2 xs:gap-6 text-sm">
                        <dt className="font-medium text-[--fg-color]">{t('podindex-configured')}</dt>
                        <dd className="text-[--fg-secondary-color]">
                            {configModel?.podindexConfigured ? (
                                <span className="material-symbols-outlined text-[--success-fg-color]">check_circle</span>
                            ) : (
                                <span className="material-symbols-outlined text-[--danger-fg-color]">block</span>
                            )}
                        </dd>


                        <dt className="font-medium text-[--fg-color]">{t('rss-feed')}</dt>
                        <dd className="text-[--fg-secondary-color]"><a className="text-[--accent-color] hover:text-[--accent-color-hover]" href={prependAPIKeyOnAuthEnabled(configModel!.rssFeed)} target="_blank" rel="noopener noreferrer">{prependAPIKeyOnAuthEnabled(configModel!.rssFeed)}</a></dd>

                        {versionInfo && (
                            <>
                                <dt className="font-medium text-[--fg-color]">{t('version')}</dt>
                                <dd className="text-[--fg-secondary-color]">{versionInfo.version}</dd>

                                <dt className="font-medium text-[--fg-color]">{t('commit')}</dt>
                                <dd className="text-[--fg-secondary-color]">{versionInfo.commit}</dd>

                                <dt className="font-medium text-[--fg-color]">{t('ci-build')}</dt>
                                <dd className="text-[--fg-secondary-color]">{versionInfo.ci}</dd>

                                <dt className="font-medium text-[--fg-color]">{t('build-date')}</dt>
                                <dd className="text-[--fg-secondary-color]">{versionInfo.time}</dd>

                                <dt className="font-medium text-[--fg-color]">{t('branch')}</dt>
                                <dd className="text-[--fg-secondary-color]">{versionInfo.ref}</dd>

                                <dt className="font-medium text-[--fg-color]">{t('os')}</dt>
                                <dd className="text-[--fg-secondary-color]">{versionInfo.os}</dd>
                            </>
                        )}
                    </dl>
                </div>
            </div>
        </>
    )
}
