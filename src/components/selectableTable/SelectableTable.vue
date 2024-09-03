<template>
	<v-data-table
		v-model:sortBy="modelSortBy"
		:items="modelItems"
		:headers="headers"
		density="compact"
		class="h-full"
		:height="100"
		:items-per-page="999999"
		:items-length="modelItems.length"
		@click.exact="handleTableClick"
		@dragleave="handleTableDragLeave"
		@dragover="handleTableDragOver"
		@drop="handleTableDrop"
	>
		<template #top>
			<slot name="toolbar"></slot>
		</template>

		<!-- Disable bottom pagination -->
		<template #bottom></template>

		<!-- No mods warning -->
		<template #no-data>
			<slot name="no-data"></slot>
			<!-- <v-icon icon="mdi-folder-alert-outline" size="x-large" /> <br />
			There are no mods in this instance. <br />
				Create a new one or import one! -->
		</template>

		<template #item="{ item, index }">
			<tr
				draggable="true"
				:class="[
					'cursor-pointer',
					'border-b',
					'border-b-gray-50',
					...dynamicRowClasses(item, index)
				]"
				:data-index="index"
				@dragstart="e => handleDragStart(e, index, item)"
				@dragenter="e => handleDragEnter(e, index)"
				@dragleave="e => handleDragLeave(e, index)"
				@dragover="e => handleDragOver(e, index)"
				@drop="e => handleDrop(e, index)"
				@click.exact="e => handleRowClick(e, item, index)"
				@click.ctrl="selectItem(item, { add: true })"
				@click.shift="selectTo(item)"
				@dblclick="e => handleRowDoubleClick(e, item, index)"
				@click.right="e => handleRowRightClick(e, item, index)"	
			>
				<td v-for="h in props.headers" :key="h.value">
					<slot
						:name="`column-${h.value}`"
						:item="item"
						:index="index">
						{{ item[h.value] }}
					</slot>
				</td>
			</tr>
		</template>
	</v-data-table>
</template>

<script setup lang="ts">
import ContextMenu from '@imengyu/vue3-context-menu';
import type { MenuItem } from '@imengyu/vue3-context-menu';
import { ref, defineEmits } from 'vue';

export interface HeaderType {
	title: string;
	value: string;
	sortable?: boolean;
	sortBy?: string;
}

export interface IContextMenu {
	items: IContextMenuItem[]
}

export interface IContextMenuItem {
	icon: string,
	label: string,
	condition?: (item: any) => boolean
	onClick: (item: any) => void | Promise<void>
}

const modelItems = defineModel<any[]>({
	required: true
});
const modelSortBy = defineModel<any[] | undefined>('sortBy');

const props = defineProps({
	headers: {
		type: undefined as unknown as PropType<HeaderType[]>,
		required: true
	},
	itemKey: {
		type: String,
		required: true
	},
	// operationalKey: {
	// 	type: String,
	// 	required: true
	// },
	allowDragging: {
		type: Boolean,
		default: true
	},
	checkered: {
		type: Boolean,
		default: true
	},
	colorClass: {
		type: String,
		default: 'blue'
	},
	contextMenu: {
		type: undefined as unknown as PropType<undefined | IContextMenu>,
		default: undefined
	}
});

const emits = defineEmits({
	onDragged: (draggingIndexes: number[], droppedAtIndex: number) => true,
	onDoubleClick: (item: any, index: number) => true,
})

const transformedItemsSortBy = computed(() => {
	const sortBy = modelSortBy.value;
	if (sortBy === undefined || sortBy.length === 0) return modelItems.value;

	const sortKey = sortBy[0].key
	const sortOrder = sortBy[0].order
	const sortedItems = [...modelItems.value]
		.sort((a, b) => {
			const aValue = a[sortKey]
			const bValue = b[sortKey]
			return sortOrder === 'desc' ? bValue - aValue : aValue - bValue
		})

	return sortedItems;
})

function getTransformedItemIndex(key: any) {
	return transformedItemsSortBy.value.findIndex(i => itemKey(i) === key);
}

const itemKey = (item: any) => {
	if (typeof item === 'undefined') return undefined;
	return item[props.itemKey];
};

const selectedItems = ref<string[]>([]);
const lastSelectedItemKey = ref<string | null>(null);

function selectItem(item: any, opts: {forced?: boolean; add?: boolean} = { forced: false, add: false }) {
	const key = itemKey(item);

	lastSelectedItemKey.value = key;
	
	if (typeof key === 'undefined') {
		console.error('Select item "key" is undefined');
		return;
	}

	if (opts.add) {
		if (selectedItems.value.includes(key)) {
			if (opts.forced) return;
			selectedItems.value = selectedItems.value.filter(si => si !== key);
		} else {
			selectedItems.value.push(key);
		}
	} else {
		if (selectedItems.value.length > 1) {
			selectedItems.value = [key];
		} else if (selectedItems.value.includes(key)) {
			if (opts.forced) return;
			selectedItems.value = [];
		} else {
			selectedItems.value = [key];
		}
	}
}

function selectTo(item: any) {
	const selectToKey = itemKey(item);

	// console.log(`Select ${lastSelectedItemKey.value} -> ${selectToKey}`)

	if (lastSelectedItemKey.value === null) {
		selectItem(item);
		return;
	}

	const indexA = getTransformedItemIndex(selectToKey);
	const indexB = getTransformedItemIndex(lastSelectedItemKey.value);

	const fromIndex = Math.min(indexA, indexB);
	const toIndex = Math.max(indexA, indexB);

	for (let i = fromIndex; i <= toIndex; i++) {
		selectItem(transformedItemsSortBy.value[i], {
			add: true,
			forced: true
		});
	}

	lastSelectedItemKey.value = selectToKey;
}

function dynamicRowClasses(item: any, index: number) {
	const classes = [];

	if (hoveringIndex.value === index) {
		classes.push(`bg-${props.colorClass}-lighten-2`);
	}

	if (props.checkered && index % 2) classes.push('bg-zinc-900');

	if (selectedItems.value.includes(itemKey(item))) {
		if (props.checkered && index % 2) classes.push(`bg-${props.colorClass}-darken-2`);
		else classes.push(`bg-${props.colorClass}`);
	}

	return classes;
}

const draggingIndex = ref<number | null>(null);
const hoveringIndex = ref<number | null>(null);

const canDrag = computed(() => {
	// return props.allowDragging && isOrderedByOperationalKey.value;
	return props.allowDragging;
})

function handleTableClick() {
	selectedItems.value = [];
	lastSelectedItemKey.value = null;
}

function handleRowClick(e: MouseEvent, item: any, index: number) {
	e.stopPropagation();
	// selectedItems.value = [itemKey(item)]
	selectItem(item);
}

function handleRowDoubleClick(event: MouseEvent, item: any, index: number) {
	emits('onDoubleClick', item, index);
}

function handleRowRightClick(e: MouseEvent, _item: any, index: number) {
	e.preventDefault();

	selectItem(_item, {
		forced: true,
		add: e.ctrlKey
	});

	// Use the current item._item if it exists
	const item = _item?._item ?? _item;

	if (typeof props.contextMenu === 'undefined') return;
	if (props.contextMenu.items.length < 1) return;

	const contextMenuItems: MenuItem[] = [];
	for (const contextMenuItem of props.contextMenu.items) {
		if (contextMenuItem.condition) {
			console.log('Condition 1:', contextMenuItem.condition);
			console.log('Condition 2:', item);
			console.log('Condition 3:', contextMenuItem.condition(item));
		}

		// Check if menu item condition is met (if any)
		if (contextMenuItem.condition && !contextMenuItem.condition(item)) continue;

		contextMenuItems.push({
			icon: contextMenuItem.icon,
			label: contextMenuItem.label,
			onClick: () => contextMenuItem.onClick(item)
		});
	}

	ContextMenu.showContextMenu({
		x: e.x,
		y: e.y,
		theme: 'flat dark',
		items: contextMenuItems
	});
}

function handleTableDragLeave(e: DragEvent) {
}

function handleTableDragOver(e: DragEvent) {
}

function handleTableDrop(e: DragEvent) {
}

function handleDragStart(e: DragEvent, index: number, item: { name: string }) {
	if (!canDrag.value) return;
	e.dataTransfer?.setData('text/plain', item.name);

	// Forcibly select item
	selectItem(item, {
		forced: true,
		add: e.ctrlKey
	});

	draggingIndex.value = index;
}

function handleDragEnter(e: DragEvent, index: number) {
	hoveringIndex.value = index;
}

function handleDragLeave(e: DragEvent, index: number) {
}

function handleDragOver(e: DragEvent, index: number) {
	if (!canDrag.value) return;
	e.preventDefault(); // Necessary, allows us to drop
}

async function handleDrop(e: DragEvent, droppedAtIndex: number) {
	if (!canDrag.value) return;
	e.preventDefault();

	if (draggingIndex.value !== null) {
		const draggingIndexes = selectedItems.value.map(selectedItemKey =>
			modelItems.value.findIndex(item => {
				return itemKey(item) === selectedItemKey;
			})
		);
		emits('onDragged', draggingIndexes, droppedAtIndex);
	}

	hoveringIndex.value = null;
	draggingIndex.value = null;
}
</script>
