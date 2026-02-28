import {FC, useState} from "react";
import useAudioPlayer from "../store/AudioPlayerSlice";
import {$api} from "../utils/http";
import {useQueryClient} from "@tanstack/react-query";
import {components} from "../../schema";

export type EditableHeadingProps = {
    initialText: string;
    allowedToEdit?: boolean;
    podcastId: number
}

export const EditableHeading: FC<EditableHeadingProps> = ({initialText, allowedToEdit, podcastId})=>{
    const [text, setText] = useState<string>(initialText);
    const updateTitleOfPodcast = $api.useMutation('put', '/api/v1/podcasts/{id}/name')
    const queryClient = useQueryClient()
    const queryClient2 = $api.useQuery('get', '/api/v1/podcasts/{id}', {
        params: {
            path: {
                id: String(podcastId)
            }
        }
    })

    return (
        <h1 onBlur={()=>{
            updateTitleOfPodcast.mutateAsync({
                params: {
                    path: {
                        id: podcastId
                    }
                },
                body: {
                    name: text
                }
            }).then(()=>{
                queryClient.setQueryData(['get', '/api/v1/podcasts/{id}',        { params: {
                    path: {
                        id: String(podcastId)
                    }
                }}], (oldData: components['schemas']['PodcastDto'])=>({...oldData, name: text}))
            })
        }} className="inline align-middle mr-2 font-bold leading-none! text-3xl xs:text-4xl ui-text" contentEditable={allowedToEdit} suppressContentEditableWarning={allowedToEdit} onInput={(event)=>{
            // @ts-ignore
            setText(event.target.textContent)
        }}>
            {initialText}
        </h1>
    );
}
