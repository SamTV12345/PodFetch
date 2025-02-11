import { FC } from 'react'
import { useTranslation } from 'react-i18next'
import { CustomButtonSecondary } from './CustomButtonSecondary'
import 'material-symbols/outlined.css'
import {client} from "../utils/http";

export const SettingsOPMLExport: FC = () => {
    const { t } = useTranslation()

    const downloadOPML = (exportType: string) => {
        client.request("get","/api/v1/settings/opml/{type_of}", {
            parseAs: "text",
            params: {
                path: {
                    type_of: exportType
                }
            }
        }).then((response) => {
            const blob = new Blob([response.data!], { type: "text/plain" })
            const href = URL.createObjectURL(blob)

            // create "a" HTML element with href to file & click
            const link = document.createElement('a')
            link.href = href
            link.setAttribute('download', 'podcast_' + exportType + '.opml') //or any other extension
            document.body.appendChild(link)
            link.click()

            // clean up "a" element & remove ObjectURL
            document.body.removeChild(link)
            URL.revokeObjectURL(href)
        })
    }

    return (
        <div className="grid grid-cols-1 xs:grid-cols-[auto_1fr] items-center justify-items-start gap-x-20 gap-y-4 xs:gap-y-6 mb-10 text-(--fg-color)">
            <span>{t('export-with-local-urls')}</span>
            <CustomButtonSecondary className="flex items-center" onClick={() => {
                downloadOPML('local')
            }}>
                <span className="material-symbols-outlined leading-[0.875rem]">download</span> {t('download')}
            </CustomButtonSecondary>

            <span>{t('export-with-online-urls')}</span>
            <CustomButtonSecondary className="flex items-center" onClick={() => {
                downloadOPML('online')
            }}>
                <span className="material-symbols-outlined leading-[0.875rem]">download</span> {t('download')}
            </CustomButtonSecondary>
        </div>
    )
}
