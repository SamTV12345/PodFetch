import { lazy } from 'react'

export const PodcastViewLazyLoad = lazy(() =>
	import('../pages/Podcasts').then((module) => {
		return { default: module.Podcasts }
	}),
)

export const PodcastDetailViewLazyLoad = lazy(() =>
	import('../pages/PodcastDetailPage').then((module) => {
		return { default: module.PodcastDetailPage }
	}),
)

export const PodcastInfoViewLazyLoad = lazy(() =>
	import('../pages/SystemInfoPage').then((module) => {
		return { default: module.SystemInfoPage }
	}),
)

export const SettingsViewLazyLoad = lazy(() =>
	import('../pages/SettingsPage').then((module) => {
		return { default: module.SettingsPage }
	}),
)

export const UserAdminViewLazyLoad = lazy(() =>
	import('../pages/UserAdminPage').then((module) => {
		return { default: module.UserAdminPage }
	}),
)

export const TimeLineViewLazyLoad = lazy(() =>
	import('../pages/Timeline').then((module) => {
		return { default: module.Timeline }
	}),
)

export const EpisodeSearchViewLazyLoad = lazy(() =>
	import('../pages/EpisodeSearchPage').then((module) => {
		return { default: module.EpisodeSearchPage }
	}),
)

export const PlaylistViewLazyLoad = lazy(() =>
	import('../pages/PlaylistDetailPage').then((module) => {
		return { default: module.PlaylistDetailPage }
	}),
)

export const HomepageViewLazyLoad = lazy(() =>
	import('../pages/Homepage').then((module) => {
		return { default: module.Homepage }
	}),
)
