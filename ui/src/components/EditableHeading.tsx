import {FC, useState} from "react";
import useAudioPlayer from "../store/AudioPlayerSlice";
import {client} from "../utils/http";

export type EditableHeadingProps = {
    initialText: string;
}

export const EditableHeading: FC<EditableHeadingProps> = ({initialText})=>{
    const [text, setText] = useState<string>(initialText);
    const setCurrentPodcast = useAudioPlayer(state => state.setCurrentPodcast)
    const currentPodcast = useAudioPlayer(state => state.currentPodcast)

    const updateTitleOfPodcast = async (newTitle: string) => {
        return await client.PUT("/api/v1/podcasts/{id}/name", {
            params: {
                path: {
                    id: currentPodcast!.id
                }
            },
            body: {
                name: newTitle
            }
        })
    }

    return (
        <h1 onBlur={()=>{
            updateTitleOfPodcast(text).then(()=>{
                currentPodcast && setCurrentPodcast({
                    ...currentPodcast,
                    name: text
                })
            })
        }} className="inline align-middle mr-2 font-bold leading-none! text-3xl xs:text-4xl text-(--fg-color)" contentEditable={true} suppressContentEditableWarning={true} onInput={(event)=>{
            // @ts-ignore
            setText(event.target.textContent)
        }}>
            {initialText}
        </h1>
    );
}
