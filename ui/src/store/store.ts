import {configureStore} from '@reduxjs/toolkit'
import {commonSlice} from "./CommonSlice";
import {opmlImportSlice} from "./opmlImportSlice";

export const store = configureStore({
    reducer: {
        common: commonSlice.reducer,
        opmlImport: opmlImportSlice.reducer,
    },
})

// Infer the `RootState` and `AppDispatch` types from the store itself
export type RootState = ReturnType<typeof store.getState>
// Inferred type: {posts: PostsState, comments: CommentsState, users: UsersState}
export type AppDispatch = typeof store.dispatch
