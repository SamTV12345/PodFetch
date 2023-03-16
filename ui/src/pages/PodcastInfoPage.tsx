import {useEffect, useState} from "react";
import axios, {AxiosResponse} from "axios";
import {apiURL} from "../utils/Utilities";
import {SysCard} from "../components/SysCard";
import {Doughnut} from "react-chartjs-2";
import {Chart as ChartJS, ArcElement, Tooltip, Legend} from "chart.js";
import {DiskModel} from "../models/DiskModel";
import {SysExtraInfo} from "../models/SysExtraInfo";
import {useTranslation} from "react-i18next";
import {Loading} from "../components/Loading";

export const PodcastInfoPage = () => {
    const [systemInfo, setSystemInfo] = useState<SysExtraInfo>()
    const gigaByte = Math.pow(10,9)
    const megaByte = Math.pow(10,6)
    const teraByte = Math.pow(10,12)
    const {t} = useTranslation()

    useEffect(()=>{
        axios.get(apiURL+"/sys/info")
            .then((response:AxiosResponse<SysExtraInfo>) => setSystemInfo(response.data))
    },[])

    if(!systemInfo){
        return <Loading/>
    }

    ChartJS.register(ArcElement, Tooltip, Legend);

    const CPUInfo = () => {
        const calcPodcastSize = ()=>{
            if(systemInfo.podcast_directory>gigaByte){
                return (systemInfo.podcast_directory/gigaByte).toFixed(2) + " GB"
            }
            else if (systemInfo.podcast_directory<gigaByte){
                return (systemInfo.podcast_directory/megaByte).toFixed(2) + " MB"
            }
        }


        return <div className="grid grid-cols-2 text-white gap-4">
            <div>{t('cpu-brand')}</div>
            <div>{systemInfo.system.global_cpu_info.brand}</div>
            <div>{t('cpu-cores')}</div>
            <div>{systemInfo.system.cpus.length}</div>
            <div>{t('podcast-size')}</div>
            <div>{calcPodcastSize()}</div>
        </div>
    }

    const calculateFreeDiskSpace = (disk: DiskModel[]) => {
        const freeSpace = disk.reduce((x, y)=>{
            return (x+(y.total_space-y.available_space))
        },0)
        const totalSpace = disk.reduce((x, y)=>{
            return (x+y.available_space)
        },0)
        return [freeSpace, totalSpace]
    }

    return <div className="p-5">
        <h1 className="text-center text-2xl font-bold">Infoseite</h1>
        <div className="grid grid-cols-3 gap-3">
        <SysCard title={t('cpu-info')} children={<CPUInfo/>}/>
            <SysCard title={t('cpu-usage')} children={<Doughnut
                options={{
                    plugins: {
                        tooltip:{
                            callbacks: {
                                label: (context) => {
                                    return context.label + ": " + context.parsed.toFixed(2) + " %"
                                }
                            }
                        }
                    }
                }
                }
                data={{
                labels: [t('used-cpu'), t('free-cpu')],
                datasets: [{
                    label: t('cpu-usage') as string,
                    data: [systemInfo.system.global_cpu_info.cpu_usage, 100-systemInfo.system.global_cpu_info.cpu_usage],
                    backgroundColor: [
                        'rgba(255, 99, 132, 0.2)',
                        'rgba(54, 162, 235, 0.2)',
                    ],
                    borderColor: [
                    'rgba(255, 99, 132, 1)',
                    'rgba(54, 162, 235, 1)',
                    ]
                }],
            }}/>}/>
            <SysCard title={t('memory-usage')} children={<Doughnut options={{
                plugins: {
                    tooltip:{
                        callbacks: {
                            label: (context) => {
                                return context.label + ": " + context.parsed.toFixed(2) + " GB"
                            }
                        }
                    }
                }
            }
            } data={{
                labels: [t('used-memory'), t('free-memory')],
                datasets: [{
                    label: t('memory-usage') as string,
                    data: [ (systemInfo.system.total_memory-systemInfo.system.free_memory)/gigaByte, systemInfo.system.free_memory/gigaByte],
                    backgroundColor: [
                        'rgba(255, 99, 132, 0.2)',
                        'rgba(54, 162, 235, 0.2)',
                    ],
                    borderColor: [
                    'rgba(255, 99, 132, 1)',
                    'rgba(54, 162, 235, 1)'
                        ]
                }]
            }}/>}/>
            <SysCard title={t('disk-usage')} children={<Doughnut
                options={
                    {
                        plugins: {
                            tooltip:{
                                callbacks: {
                                    label: (context) => {
                                        if(context.parsed > teraByte){
                                            return context.label + ": " + (context.parsed/teraByte).toFixed(2) + " TB"
                                        }
                                        else{
                                            return context.label + ": " + (context.parsed/gigaByte).toFixed(2) + " GB"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                data={{
                labels: [t('used-disk'), t('free-disk')],
                datasets: [{
                    label: t('disk-usage') as string,
                    data: calculateFreeDiskSpace(systemInfo.system.disks),
                    backgroundColor: [
                        'rgba(255, 99, 132, 0.2)',
                        'rgba(54, 162, 235, 0.2)',
                    ],
                }]
            }}/>}/>
        </div>
    </div>
}
