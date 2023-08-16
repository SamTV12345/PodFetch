import { FC } from 'react'
import { useTranslation } from 'react-i18next'
import { AddTypes } from '../models/AddTypes'
import { ConfigModel } from '../models/SysInfo'

type AddHeaderProps = {
    selectedSearchType: AddTypes;
    setSelectedSearchType: (type: AddTypes) => void,
    configModel: ConfigModel|undefined
}

export const AddHeader: FC<AddHeaderProps> = ({ selectedSearchType,setSelectedSearchType, configModel }) => {
    const {t} = useTranslation()

    return (
        <ul className="flex flex-wrap gap-2 border-b border-[--border-color] mb-6 text-[--fg-secondary-color]">
            <li className={`cursor-pointer inline-block px-2 py-4 ${selectedSearchType === 'itunes' && 'border-b-2 border-[--accent-color] text-[--accent-color]'}`} onClick={() => setSelectedSearchType(AddTypes.ITUNES)}>
                iTunes
            </li>
            {configModel?.podindexConfigured && <li className={`cursor-pointer inline-block px-2 py-4 ${selectedSearchType === 'podindex' && 'border-b-2 border-[--accent-color] text-[--accent-color]'}`} onClick={() => setSelectedSearchType(AddTypes.PODINDEX)}>
                PodIndex
            </li>}
            <li className={`cursor-pointer inline-block px-2 py-4 ${selectedSearchType === 'opml' && 'border-b-2 border-[--accent-color] text-[--accent-color]'}`} onClick={() => setSelectedSearchType(AddTypes.OPML)}>
                {t('opml-file')}
            </li>
            <li className={`cursor-pointer inline-block px-2 py-4 ${selectedSearchType === 'feed' && 'border-b-2 border-[--accent-color] text-[--accent-color]'}`} onClick={() => setSelectedSearchType(AddTypes.FEED)}>
                {t('rss-feed-url')}
            </li>
        </ul>
    )
}
