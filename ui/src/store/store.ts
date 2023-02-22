import {configureStore} from '@reduxjs/toolkit'
import {commonSlice} from "./CommonSlice";
import {AudioPlayerSlice} from "./AudioPlayerSlice";
import {modalSlice} from "./ModalSlice";

export const store = configureStore({
    reducer: {
        common: commonSlice.reducer,
        audioPlayer: AudioPlayerSlice.reducer,
        modal: modalSlice.reducer
    },
})

// Infer the `RootState` and `AppDispatch` types from the store itself
export type RootState = ReturnType<typeof store.getState>
// Inferred type: {posts: PostsState, comments: CommentsState, users: UsersState}
export type AppDispatch = typeof store.dispatch
