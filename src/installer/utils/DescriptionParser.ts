export function parseDescription(rawString: string) {
	return (
		rawString
			.replaceAll('\r\n', '\n')
			.replaceAll('\n\n', '\n')
			.replaceAll('\n', '<br />')
			// Replace urls, with clickable links
			.replaceAll(
				/https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)/g,
				'<a class="text-blue-300 hover:underline" onclick="(function(e) { e.preventDefault(); e.stopPropagation(); return window.__TAURI__.shell.open(\'$&\') })(arguments[0]);return false;">$&</a>'
			)
	);
}
