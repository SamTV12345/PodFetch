import {FC, useMemo} from "react";
import {CirclePlay} from "lucide-react";
import {$api} from "../utils/http";
import {t} from "i18next";
import {components} from "../../schema";
import {startAudioPlayer} from "../utils/audioPlayer";
import useCommon from "../store/CommonSlice";
import useAudioPlayer from "../store/AudioPlayerSlice";

type PodcastEpisodeChapterTableProps = {
    podcastEpisode: components["schemas"]["PodcastEpisodeDto"],
    className?: string
}

export const PodcastEpisodeChapterTable: FC<PodcastEpisodeChapterTableProps> = ({podcastEpisode, className}) => {
    const setSelectedEpisodes = useCommon(state=>state.setSelectedEpisodes)
    const setCurrentEpisodeIndex = useAudioPlayer(state=>state.setCurrentPodcastEpisode)
    const chapters = $api.useQuery('get', '/api/v1/podcasts/episodes/{id}/chapters', {
        params: {
            path: {
                id: podcastEpisode.id
            }
        }
    })
    const currentPodcast = $api.useQuery('get', '/api/v1/podcasts/reverse/episode/{id}', {
        params: {
            path: {
                id: podcastEpisode.id
            }
        }
    })

    const timeslotDisplay = useMemo(()=>{
        if(!chapters.data){
            return []
        }
        return chapters.data.map(chapter=>{
            const start_minutes = chapter.startTime / 60 >= 1 ? `${Math.floor(chapter.startTime / 60)}:${(chapter.startTime % 60).toString().padStart(2, '0')}` : `0:${chapter.startTime.toString().padStart(2, '0')}`
            const end_minutes = chapter.endTime / 60 >= 1 ? `${Math.floor(chapter.endTime / 60)}:${(chapter.endTime % 60).toString().padStart(2, '0')}` : `0:${chapter.endTime.toString().padStart(2, '0')}`

            return `${start_minutes} - ${end_minutes}min`
        })
    }, [chapters])


  return <div className={className}>
      <table className="text-left text-sm ui-text w-full">
        <thead>
        <tr className="border-b ui-border">
            <th scope="col" className="pr-2 py-3">
                {t('title')}
            </th>
        <th scope="col" className="pr-2 py-3  hidden sm:table-cell">
            {t('timeslot')}
        </th>
    <th scope="col" className="px-2 py-3">
        {t('actions')}
    </th>
</tr>
</thead>
    <tbody>
    {chapters.data?.map((chapter, index)=><tr className="border-b ui-border" key={chapter.id}>
        <td className="pr-2 py-4 break-words">
            {chapter.title}
        </td>
        <td className="pr-2 py-4 hidden sm:table-cell">
            {timeslotDisplay[index]}
        </td>
        <td className="pr-2 py-4">
            <CirclePlay
                size={32}
                fill="currentColor"
                className="cursor-pointer ui-text hover:ui-text-hover active:scale-90"
                onClick={async (e) => {
                    e.stopPropagation()
                    setSelectedEpisodes([{
                        podcastEpisode,
                        podcastHistoryItem: null
                    }])
                    setCurrentEpisodeIndex(0)
                    await startAudioPlayer(podcastEpisode.local_url, chapter.startTime ?? 0)
                }}
            />
        </td>
    </tr>)}
    </tbody>
  </table>
  </div>
}
