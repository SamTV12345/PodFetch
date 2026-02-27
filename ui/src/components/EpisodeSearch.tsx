import { FC, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {formatTime, removeHTML} from '../utils/Utilities'
import { useDebounce } from '../utils/useDebounce'
import { CustomInput } from './CustomInput'
import { Spinner } from './Spinner'
import { EmptyResultIcon } from '../icons/EmptyResultIcon'
import {$api} from "../utils/http";
import {components} from "../../schema";

type EpisodeSearchProps = {
    classNameResults?: string,
    onClickResult?: (e: components["schemas"]["PodcastEpisodeDto"]) => void,
    resultsMaxHeight?: string,
    showBlankState?: boolean
}

export const EpisodeSearch: FC<EpisodeSearchProps> = ({ classNameResults = '', onClickResult = () => null,
                                                          resultsMaxHeight = 'none',
                                                          showBlankState = true }) => {
    const [searching, setSearching] = useState<boolean>()
    const [searchName, setSearchName] = useState<string>('')
    const [searchResults, setSearchResults] = useState<components["schemas"]["PodcastEpisodeDto"][]>([])
    const searchEpisodesMutation = $api.useMutation('get', '/api/v1/podcasts/{podcast}/query')
    const { t } = useTranslation()
    const {data, isLoading} = $api.useQuery('get', '/api/v1/users/{username}', {
        params: {
            path: {
                username: 'me'
            }
        }
    })



    const performSearch = () => {
        if (searchName.trim().length > 0) {
            setSearching(true)

            searchEpisodesMutation.mutateAsync({
                params: {
                    path: {
                        podcast: searchName
                    }
                }
            }).then((v) => {
                setSearchResults(v ?? [])
                setSearching(false)
            })
        }
    }

    useDebounce(performSearch, 500, [searchName])

    if (isLoading || !data) {
        return <div className="grid place-items-center p-10">
            <Spinner className="w-12 h-12"/>
        </div>
    }

    return (
        <>
            {/* Search field */}
            <div className="flex items-center relative">
                <CustomInput className="pl-10 w-full" id="search-input" onChange={(v) =>
                    setSearchName(v.target.value)} placeholder={t('search-episodes')!} type="text" value={searchName} />

                <span className="material-symbols-outlined absolute left-2 text-(--input-icon-color)">search</span>
            </div>

            {/* Results */
            searching ? (
                <div className="grid place-items-center p-6">
                    <Spinner className="w-12 h-12"/>
                </div>
            ) : searchResults.length === 0 ? (
                <div className="grid place-items-center py-4">
                    {searchName ? (
                        <span className="p-3 text-(--fg-secondary-color)">{t('no-results-found-for')} "<span className="text-(--fg-color)">{searchName}</span>"</span>
                    ) : (
                        showBlankState && <EmptyResultIcon />
                    )}
                </div>
            ) : (
                <ul
                    className={`flex min-w-0 flex-col gap-10 overflow-y-auto overflow-x-hidden my-4 px-6 py-6 scrollbox-y ${classNameResults}`}
                    style={{maxHeight: resultsMaxHeight}}
                >
                    {searchResults.map((episode, i) => (
                        <li className="flex min-w-0 min-h-24 gap-4 cursor-pointer group overflow-hidden" key={i} onClick={() => {
                            onClickResult(episode)
                        }}>
                            {/* Thumbnail */}
                            <img alt={episode.name} className="
                                hidden xs:block
                                shrink-0
                                rounded-lg w-32 transition-shadow group-hover:shadow-[0_4px_32px_rgba(0,0,0,0.3)]
                            " src={episode.image_url} />

                            {/* Information */}
                            <div className="flex min-w-0 flex-col gap-2">
                                <span className="text-sm text-(--fg-secondary-color)">{formatTime(episode.date_of_recording)}</span>
                                <span className="font-bold leading-tight text-(--fg-color) transition-color group-hover:text-(--fg-color-hover) break-words [overflow-wrap:anywhere]">{episode.name}</span>
                                <div className="leading-[1.75] line-clamp-3 text-sm text-(--fg-color) break-words [overflow-wrap:anywhere]" dangerouslySetInnerHTML={removeHTML(episode.description)}></div>
                            </div>
                        </li>
                    ))}
                </ul>
            )}
        </>
    )
}
