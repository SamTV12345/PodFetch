import {createSlice, PayloadAction} from '@reduxjs/toolkit'


// Define a type for the slice state
interface ModalProps {
    openModal:boolean,
    openAddModal: boolean
}

// Define the initial state using that type
const initialState: ModalProps = {
    openModal: false,
    openAddModal: false,
}

export const modalSlice = createSlice({
    name: 'modalSlice',
    initialState,
    reducers: {
        setModalOpen: (state, action)=>{
            state.openModal = action.payload
        },
        setOpenAddModal: (state, action)=>{
            state.openAddModal = action.payload
        }
    }
})

export const {setModalOpen, setOpenAddModal} = modalSlice.actions

export default modalSlice.reducer
