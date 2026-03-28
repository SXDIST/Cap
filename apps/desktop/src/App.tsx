import { Route, Router, useCurrentMatches } from "@solidjs/router";
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import {
	getCurrentWebviewWindow,
	type WebviewWindow,
} from "@tauri-apps/api/webviewWindow";
import { message } from "@tauri-apps/plugin-dialog";
import { createEffect, onCleanup, onMount, Suspense } from "solid-js";
import { Toaster } from "solid-toast";

import "@cap/ui-solid/main.css";
import "unfonts.css";
import "./styles/theme.css";

import { CapErrorBoundary } from "./components/CapErrorBoundary";
import WindowChromeLayout from "./routes/(window-chrome)";
import NewMainPage from "./routes/(window-chrome)/new-main";
import OnboardingPage from "./routes/(window-chrome)/onboarding";
import SettingsLayout from "./routes/(window-chrome)/settings";
import SettingsFeedbackPage from "./routes/(window-chrome)/settings/feedback";
import SettingsGeneralPage from "./routes/(window-chrome)/settings/general";
import SettingsHotkeysPage from "./routes/(window-chrome)/settings/hotkeys";
import SettingsIntegrationsPage from "./routes/(window-chrome)/settings/integrations";
import SettingsS3ConfigPage from "./routes/(window-chrome)/settings/integrations/s3-config";
import SettingsLicensePage from "./routes/(window-chrome)/settings/license";
import SettingsRecordingsPage from "./routes/(window-chrome)/settings/recordings";
import SettingsScreenshotsPage from "./routes/(window-chrome)/settings/screenshots";
import SettingsChangelogPage from "./routes/(window-chrome)/settings/changelog";
import SettingsExperimentalPage from "./routes/(window-chrome)/settings/experimental";
import SettingsTranscriptionPage from "./routes/(window-chrome)/settings/transcription";
import UpdatePage from "./routes/(window-chrome)/update";
import UpgradePage from "./routes/(window-chrome)/upgrade";
import CameraPage from "./routes/camera";
import CaptureAreaPage from "./routes/capture-area";
import DebugPage from "./routes/debug";
import EditorPage from "./routes/editor";
import InProgressRecordingPage from "./routes/in-progress-recording";
import ModeSelectPage from "./routes/mode-select";
import NotificationsPage from "./routes/notifications";
import RecordingsOverlayPage from "./routes/recordings-overlay";
import ScreenshotEditorPage from "./routes/screenshot-editor";
import { generalSettingsStore } from "./store";
import TargetSelectOverlayPage from "./routes/target-select-overlay";
import WindowCaptureOccluderPage from "./routes/window-capture-occluder";
import { initAnonymousUser } from "./utils/analytics";
import { type AppTheme, commands } from "./utils/tauri";
import titlebar from "./utils/titlebar-state";

const queryClient = new QueryClient({
	defaultOptions: {
		queries: {
			refetchOnWindowFocus: false,
			refetchOnReconnect: false,
		},
		mutations: {
			onError: (e) => {
				message(`Error\n${e}`);
			},
		},
	},
});

export default function App() {
	return (
		<QueryClientProvider client={queryClient}>
			<Suspense>
				<Inner />
			</Suspense>
		</QueryClientProvider>
	);
}

function Inner() {
	const currentWindow = getCurrentWebviewWindow();
	createThemeListener(currentWindow);

	onMount(() => {
		initAnonymousUser();
	});

	return (
		<>
			<Toaster
				position="bottom-right"
				containerStyle={{
					"margin-top": titlebar.height,
				}}
				toastOptions={{
					duration: 3500,
					style: {
						padding: "8px 16px",
						"border-radius": "15px",
						"border-color": "var(--gray-200)",
						"border-width": "1px",
						"font-size": "1rem",
						"background-color": "var(--gray-50)",
						color: "var(--text-secondary)",
					},
				}}
			/>
			<CapErrorBoundary>
				<Router
					root={(props) => {
						const matches = useCurrentMatches();

						onMount(() => {
							for (const match of matches()) {
								if (match.route.info?.AUTO_SHOW_WINDOW === false) return;
							}

							if (location.pathname !== "/camera") currentWindow.show();
						});

						return <Suspense fallback={null}>{props.children}</Suspense>;
					}}
				>
					<Route path="/" component={WindowChromeLayout}>
						<Route path="/" component={NewMainPage} />
						<Route path="/settings" component={SettingsLayout}>
							<Route path="/" component={SettingsGeneralPage} />
							<Route path="/general" component={SettingsGeneralPage} />
							<Route path="/recordings" component={SettingsRecordingsPage} />
							<Route
								path="/transcription"
								component={SettingsTranscriptionPage}
							/>
							<Route path="/screenshots" component={SettingsScreenshotsPage} />
							<Route path="/hotkeys" component={SettingsHotkeysPage} />
							<Route path="/changelog" component={SettingsChangelogPage} />
							<Route path="/feedback" component={SettingsFeedbackPage} />
							<Route
								path="/experimental"
								component={SettingsExperimentalPage}
							/>
							<Route path="/license" component={SettingsLicensePage} />
							<Route
								path="/integrations"
								component={SettingsIntegrationsPage}
							/>
							<Route
								path="/integrations/s3-config"
								component={SettingsS3ConfigPage}
							/>
						</Route>
						<Route path="/onboarding" component={OnboardingPage} />
						<Route path="/upgrade" component={UpgradePage} />
						<Route path="/update" component={UpdatePage} />
					</Route>
					<Route path="/camera" component={CameraPage} />
					<Route path="/capture-area" component={CaptureAreaPage} />
					<Route path="/debug" component={DebugPage} />
					<Route path="/editor" component={EditorPage} />
					<Route
						path="/in-progress-recording"
						component={InProgressRecordingPage}
					/>
					<Route path="/mode-select" component={ModeSelectPage} />
					<Route path="/notifications" component={NotificationsPage} />
					<Route path="/recordings-overlay" component={RecordingsOverlayPage} />
					<Route path="/screenshot-editor" component={ScreenshotEditorPage} />
					<Route
						path="/target-select-overlay"
						component={TargetSelectOverlayPage}
					/>
					<Route
						path="/window-capture-occluder"
						component={WindowCaptureOccluderPage}
					/>
				</Router>
			</CapErrorBoundary>
		</>
	);
}

function createThemeListener(currentWindow: WebviewWindow) {
	const generalSettings = generalSettingsStore.createQuery();

	createEffect(() => {
		update(generalSettings.data?.theme ?? null);
	});

	onMount(async () => {
		let unlistenThemeChanged: (() => void) | undefined;

		onCleanup(() => {
			unlistenThemeChanged?.();
		});

		const unlisten = await currentWindow.onThemeChanged((_) =>
			update(generalSettings.data?.theme),
		);
		unlistenThemeChanged = unlisten ?? undefined;
	});

	function update(appTheme: AppTheme | null | undefined) {
		if (location.pathname === "/camera") return;

		if (appTheme === undefined || appTheme === null) return;

		const isDark =
			appTheme === "dark" ||
			(appTheme === "system" &&
				window.matchMedia("(prefers-color-scheme: dark)").matches);

		try {
			if (appTheme === "system") {
				localStorage.removeItem("cap-theme");
			} else {
				localStorage.setItem("cap-theme", appTheme);
			}
		} catch {}

		commands.setTheme(appTheme).then(() => {
			document.documentElement.classList.toggle("dark", isDark);
		});
	}
}
