import {createSlice, PayloadAction} from "@reduxjs/toolkit";

interface OpmlImportProps {
    progress: boolean[],
    messages: string[],
    inProgress: boolean
}

const initialState: OpmlImportProps = {
    progress: [],
    messages:[],
    inProgress: false,
}
export const opmlImportSlice = createSlice({
    name: 'opmlImportSlice',
    initialState,
    reducers: {
        setProgress: (state, action:PayloadAction<boolean[]>) => {
            state.progress = action.payload
        },
        setMessages: (state, action:PayloadAction<string[]>) => {
            state.messages = action.payload
        },
        setInProgress: (state, action:PayloadAction<boolean>) => {
            state.inProgress = action.payload
        }
    }
})


export const {setProgress,setInProgress,setMessages} = opmlImportSlice.actions

export default opmlImportSlice.reducer
