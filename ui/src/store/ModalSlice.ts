// Define a type for the slice state
import { create } from 'zustand'

interface ModalProps {
	openModal: boolean
	openAddModal: boolean
	setOpenModal: (openModal: boolean) => void
	setOpenAddModal: (openAddModal: boolean) => void
}

const useModal = create<ModalProps>((set, _get) => ({
	openModal: false,
	openAddModal: false,
	setOpenModal: (openModal: boolean) => set({ openModal }),
	setOpenAddModal: (openAddModal: boolean) => set({ openAddModal }),
}))

export default useModal
