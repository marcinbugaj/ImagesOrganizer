import * as React from 'react';
import './App.css';
import { MapContainer, TileLayer, Marker, useMap, Polyline, useMapEvent, useMapEvents } from 'react-leaflet';
import { LatLngBoundsLiteral, LatLngExpression } from 'leaflet';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import ChevronRightIcon from '@mui/icons-material/ChevronRight';
import { TreeView } from '@mui/x-tree-view/TreeView';
import { TreeItem } from '@mui/x-tree-view/TreeItem';
import { Button, Checkbox, Container, Dialog, DialogActions, DialogContent, DialogContentText, DialogTitle, FormControlLabel, FormGroup, Grid, ImageList, ImageListItem, LinearProgress, TextField } from '@mui/material';
import { CheckBox } from '@mui/icons-material';

interface Node {
  convex_hull: Coord[]
  number_of_leaves: number
  children: Tree[]

  center?: Coord
}

interface Coord {
  lat: number,
  lon: number
}

interface Leaf {
  filepath: string
  coord: Coord
}
interface Tree {
  node?: Node
  leaf?: Leaf

  id?: string
}

type Clusters = Array<Array<string>>

type Commit = {
  clusters: Clusters
  folder: string
  dryrun: boolean
}

function getId(tree: Tree) {
  if (tree.id == undefined) {
    tree.id = uuidv4();
  }

  return tree.id;
}

const position: LatLngExpression = [51.505, -0.09]

function getCenterCoord(tree: Tree): Coord {
  if (tree.leaf != undefined) {
    return tree.leaf.coord
  } else if (tree.node != undefined) {
    if (tree.node.center == undefined) {
      let coord = tree.node.convex_hull.reduce(
        (prev, curr, _a, _b) => {
          return { lat: prev.lat + curr.lat, lon: prev.lon + curr.lon };
        })
      let num = tree.node.convex_hull.length;
      let coord1 = { lat: coord.lat / num, lon: coord.lon / num };
      tree.node.center = coord1;
    }
    return tree.node.center;
  }

  return { lat: 0, lon: 0 };
}

function deepCopyMyTreeView(treeView: MyTreeView): MyTreeView {
  const copiedChildren = treeView.children.map(deepCopyMyTreeView);

  return {
    tree: treeView.tree,
    children: copiedChildren
  };
}

type MyTreeView = {
  tree: Tree
  children: Array<MyTreeView>
  // if tree is Node then `children` is a subarray of tree.node.children
  // if tree is Leaf then `children` is empty
}

function buildMap(tree: Tree) {
  let map = new Map<string, Tree>();

  let toProcess = [tree];

  while (toProcess.length != 0) {
    const elem = toProcess.pop();

    if (elem != undefined) {
      map.set(getId(elem), elem);

      if (elem.node != undefined) {
        elem.node.children.forEach(child => toProcess.push(child));
      }
    }
  }

  return map;
}

function getTreeViewLeaves(treeView: MyTreeView): MyTreeView[] {
  if (treeView.tree.leaf != undefined) {
    return [treeView];
  } else if (treeView.tree.node != undefined) {
    if (treeView.children.length == 0) {
      return [treeView]
    } else {
      return treeView
        .children
        .map(tv => getTreeViewLeaves(tv))
        .flat();
    }
  }

  return [];
}

function getAllLeaves(tree: Tree): Array<Leaf> {
  if (tree.leaf != undefined) {
    return [tree.leaf];
  } else if (tree.node != undefined) {
    return tree.node.children.flatMap(getAllLeaves);
  }

  return [];
}

function getTreeViewNodeTypeLeaves(treeView: MyTreeView): Array<Array<Leaf>> {
  if (treeView.tree.leaf != undefined) {
    return [];
  } else if (treeView.tree.node != undefined) {
    if (treeView.children.length == 0) {
      return [getAllLeaves(treeView.tree)];
    } else {
      return treeView
        .children
        .flatMap(getTreeViewNodeTypeLeaves);
    }
  }

  return [];
}

function getClusters(treeView: MyTreeView): Clusters {
  return getTreeViewNodeTypeLeaves(treeView).map(leavesArray => leavesArray.map(leaf => leaf.filepath));
}

interface MyTreeItemProps {
  idd: string
  lbl: string
  tr: Tree
  iChangedCb: (content: React.JSX.Element) => void
  iChangedAsTreeViewCb: (treeView: MyTreeView) => void
}

interface MyTreeViewProps {
  iChangedAsTreeViewCb: (treeView: MyTreeView) => void
  onNodeSelect: (id: string) => void
  tree: Tree
}

function getLabel(tree: Tree): string {
  if (tree.node != undefined) {
    return tree.node.number_of_leaves.toString();
  } else if (tree.leaf != undefined) {
    return tree.leaf.filepath;
  }
  return ""
}

function uuidv4() {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'
    .replace(/[xy]/g, function (c) {
      const r = Math.random() * 16 | 0,
        v = c == 'x' ? r : (r & 0x3 | 0x8);
      return v.toString(16);
    });
}

function partition(l: Tree[]): { leaves: Tree[], nodes: Tree[] } {
  let leaves: Tree[] = [];
  let nodes: Tree[] = [];
  l.forEach(elem => {
    if (elem.leaf != undefined) {
      leaves.push(elem);
    } else if (elem.node != undefined) {
      nodes.push(elem);
    }
  });

  return { leaves, nodes };
}

function MyTreeItem({ idd, lbl, tr, iChangedCb, iChangedAsTreeViewCb }: MyTreeItemProps) {
  const nonExpanded = <TreeItem nodeId={idd} label={lbl} onDoubleClick={onExpand}></TreeItem>;

  function onCollapse(event: React.MouseEvent<HTMLElement>) {
    event.stopPropagation();

    iChangedCb(nonExpanded);
    iChangedAsTreeViewCb({ tree: tr, children: [] });
  }

  function onExpand(event: React.MouseEvent<HTMLElement>) {
    event.stopPropagation();

    if (tr.node != undefined) {
      let children: React.JSX.Element[] = [];
      let treeView: MyTreeView = { tree: tr, children: [] };

      const { leaves, nodes } = partition(tr.node.children);

      let index: number = 0;
      let handle = (tree: Tree) => {
        const i = index;
        let element = <MyTreeItem
          idd={getId(tree)}
          lbl={getLabel(tree)}
          tr={tree}
          iChangedAsTreeViewCb={(updated) => {
            treeView.children[i] = updated;
            iChangedAsTreeViewCb(treeView);
          }}
          iChangedCb={(content) => {
            children[i] = content;
            iChangedCb(<TreeItem nodeId={idd} label={lbl} onDoubleClick={onCollapse}> {children} </TreeItem>);
          }}></MyTreeItem>

        children.push(element);
        treeView.children.push({ tree: tree, children: [] });
        index++;
      };
      leaves.forEach(handle);
      nodes.forEach(handle);

      let expanded = <TreeItem nodeId={idd} label={lbl} onDoubleClick={onCollapse}>{children}</TreeItem>;
      iChangedCb(expanded);
      iChangedAsTreeViewCb(treeView);
    }
  }

  return nonExpanded;
}

function MyTreeView({ tree, iChangedAsTreeViewCb, onNodeSelect }: MyTreeViewProps) {
  const [simplifiedContent, setSimplifiedContent] = React.useState(<></>);
  const [isSimplified, setIsSimplified] = React.useState(false);

  function treeViewChanged(treeView: MyTreeView) {
    const copied = deepCopyMyTreeView(treeView);

    const clusters = getTreeViewLeaves(copied).flatMap(treeView => {
      if (treeView.tree.node != undefined) {
        return [treeView.tree];
      }
      return [];
    });

    const simplifiedContent = clusters.map(tree => {
      return (<TreeItem nodeId={getId(tree)} label={getLabel(tree)}></TreeItem>);
    })

    setSimplifiedContent(<>{simplifiedContent}</>);

    iChangedAsTreeViewCb(copied);
  }

  const initialContent = <MyTreeItem
    idd={getId(tree)}
    lbl={getLabel(tree)}
    tr={tree}
    iChangedAsTreeViewCb={treeViewChanged}
    iChangedCb={cb}></MyTreeItem>;

  const [content, setContent] = React.useState(initialContent);
  const [key, setKey] = React.useState(0);

  function cb(c: React.JSX.Element) {
    setKey(currentKey => currentKey + 1)
    setContent(c);
  }

  const handleChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setIsSimplified(event.target.checked);
  };

  let actualContent = isSimplified ? simplifiedContent : content;

  return (<>
    <FormGroup>
      <FormControlLabel
        control=
        {<Checkbox
          checked={isSimplified}
          onChange={handleChange}
        />}
        label="Show final clusters only" />
    </FormGroup>
    <TreeView
      className='treeView'
      aria-label="file system navigator"
      defaultCollapseIcon={<ExpandMoreIcon />}
      defaultExpandIcon={<ChevronRightIcon />}
      onNodeSelect={(event, nodeId) => onNodeSelect(nodeId)}
      sx={{ height: 240, flexGrow: 1, overflowY: 'auto' }}
    >
      <>
        <div key={key}>
          {actualContent}
        </div>
      </>
    </TreeView>
  </>
  )
}


interface MyComponentProps {
  selectedTree: Tree | undefined
  treeView: MyTreeView | undefined
  onMoveEnd: () => void
}

interface AppProps {
  tree: Tree
  onCommit: (treeView: MyTreeView) => void
}

interface FolderSelectionDialogProps {
  onChosen: (folder: string) => void
}

interface ErrorMessageDialogProps {
  message: string
}

interface ImageViewerProps {
  selectedTree: Tree
}

function getBoundsForConvexFull(convex_hull: Coord[]): LatLngBoundsLiteral {
  const lats = convex_hull.map(coord => coord.lat);
  const lons = convex_hull.map(coord => coord.lon);

  const minLat = Math.min(...lats);
  const maxLat = Math.max(...lats);
  const minLon = Math.min(...lons);
  const maxLon = Math.max(...lons);

  return [[minLat, minLon], [maxLat, maxLon]];
}

function convexHullToPolyline(ch: Coord[]): LatLngExpression[] {
  return ch.map(c => { return { lat: c.lat, lng: c.lon } });
}

function MyComponent({ selectedTree, treeView, onMoveEnd }: MyComponentProps) {
  const map = useMap();

  const purpleOptions = { color: 'purple' }
  const limeOptions = { color: 'lime' }

  useMapEvents({
    moveend() {
      onMoveEnd();
    },
  })

  let actualMarkers: React.JSX.Element[] = [];
  let convexHulls: React.JSX.Element[] = [];
  if (treeView != undefined) {
    let coords = getTreeViewLeaves(treeView);
    coords.forEach(myTreeView => {
      let m = getCenterCoord(myTreeView.tree)
      let coord = { lat: m.lat, lng: m.lon };
      actualMarkers.push(<Marker position={coord}></Marker>);

      if (myTreeView.tree.node != undefined) {
        let polyline = convexHullToPolyline(myTreeView.tree.node.convex_hull);
        convexHulls.push(<Polyline pathOptions={purpleOptions} positions={polyline} />)
      }
    });
  }

  if (selectedTree != undefined) {
    if (selectedTree.node != undefined) {
      const convex_hull = selectedTree.node.convex_hull

      const polyline = convexHullToPolyline(convex_hull);

      map.flyToBounds(getBoundsForConvexFull(convex_hull));

      return (
        <>
          <Polyline pathOptions={limeOptions} positions={polyline} />
          {actualMarkers}
          {convexHulls}
        </>
      );
    } else {
      const coord = getCenterCoord(selectedTree);
      map.flyTo({ lat: coord.lat, lng: coord.lon }, map.getZoom());
    }
  }

  return <>{actualMarkers}{convexHulls}</>
}

function ErrorMessageDialog({ message }: ErrorMessageDialogProps) {
  const content = message + " " + "Reload the page."
  return (
    <Dialog
      open={true}
    >
      <DialogTitle>Error</DialogTitle>
      <DialogContent>
        <DialogContentText>
          {content}
        </DialogContentText>
      </DialogContent>
    </Dialog>
  );
}

function FolderSelectionDialog({ onChosen }: FolderSelectionDialogProps) {
  const [dialogOpen, setDialogOpen] = React.useState(true);

  let circularProgress = dialogOpen ? <></> : <LinearProgress />;

  const handleClose = () => {
    setDialogOpen(false);
  };

  return (
    <>
      <Dialog
        open={dialogOpen}
        onClose={handleClose}
        PaperProps={{
          component: 'form',
          onSubmit: async (event: React.FormEvent<HTMLFormElement>) => {
            event.preventDefault();
            const formData = new FormData(event.currentTarget);
            const formJson = Object.fromEntries((formData as any).entries());
            const directory = formJson.directory;

            onChosen(directory);

            handleClose();
          },
        }}
      >
        <DialogTitle>Select directory</DialogTitle>
        <DialogContent>
          <DialogContentText>
            Absolute path to a directory where images are searched for recursively
          </DialogContentText>
          <TextField
            autoFocus
            required
            margin="dense"
            id="name"
            name="directory"
            label="Absolute path to directory"
            type="text"
            fullWidth
            variant="standard"
          />
        </DialogContent>
        <DialogActions>
          <Button type="submit">Ok</Button>
        </DialogActions>
      </Dialog>
      {circularProgress}
    </>
  );
}

type Context = {
  tree: Tree,
  folder: string
}

export default function App() {
  const [ctx, setCtx] = React.useState<Context | undefined>(undefined);
  const [errorMessage, setErrorMessage] = React.useState<string | undefined>(undefined);

  async function onChosen(folder: string) {
    const response = await fetch(`/compute_clusters`, {
      method: "POST",
      body: folder
    });

    if (!response.ok) {
      console.log("\"compute_clusters\" failed");
      const msg = await response.text();
      setErrorMessage(msg);
    } else {
      const json = await response.json();
      const tree = json as Tree;
      setCtx({
        tree,
        folder
      });
    }
  }

  async function onCommit(treeView: MyTreeView) {
    if (ctx == undefined) {
      return;
    }

    let clusters = getClusters(treeView);

    console.log("on log clusters no: " + clusters.length.toString());

    let reorganizeWith = async (commit: Commit) => {
      let asJsonString = JSON.stringify(commit);
      const response = await fetch(`/reorganize`, {
        method: "POST",
        body: asJsonString
      });
      if (!response.ok) {
        console.log("\"reorganize\" failed");
        const msg = await response.text();
        setErrorMessage(msg);
        return false;
      } else {
        const responseText = await response.text();
        console.log("Reponse from reorganize: " + responseText);
        return true;
      }
    }

    let commit: Commit = {
      clusters: clusters,
      folder: ctx.folder,
      dryrun: true
    }

    if (await reorganizeWith(commit)) {
      commit.dryrun = false;
      if (await reorganizeWith(commit)) {
        setCtx(undefined);
      } else {
        setErrorMessage("File operations failed although dry run passed.")
      }
    }
  }

  const ui =
    ctx == undefined ? <></> : <UIActive onCommit={onCommit} tree={ctx.tree}></UIActive>;
  const dialog =
    ctx != undefined ? <></> : <FolderSelectionDialog onChosen={onChosen}></FolderSelectionDialog>;
  const errorDialog =
    errorMessage == undefined ? <></> : <ErrorMessageDialog message={errorMessage}></ErrorMessageDialog>;

  return (
    <Container>
      {ui}
      {dialog}
      {errorDialog}
    </Container>
  );
}

function getImg(filepath: string): React.JSX.Element {
  let url = "/file/" + filepath;
  let url2 = encodeURI(url);

  return (
    <ImageListItem key={filepath}>
      <img
        src={url2}
        loading="lazy"
      />
    </ImageListItem>
  )
}

type Path = { children: Tree[], index: number }[];

// if tree is a node the path is not empty
// if tree is a leaf the path is empty
function getPathToFirst(tree: Tree): Path {
  const path: Path = [];

  let c = tree;
  while (c.node != undefined) {
    path.push({ children: c.node.children, index: 0 });

    c = c.node.children[0];
  }

  return path;
}

function deepCopyPath(path: Path): Path {
  return path.map(e => { return { children: e.children, index: e.index }; });
}

// path may point to an arbitrary element in a tree: leaf or node
// it returns path to next leaf or empty if no leaves left
// if the path points to a node the function assumes that
// there are no more leaves down the tree
function findNext(path: Path): Path {
  if (path.length == 0) {
    return [];
  }

  let copied = deepCopyPath(path);

  const last = copied[copied.length - 1];

  let index = last.index + 1;
  const len = last.children.length;

  while (index < len) {
    const tree = last.children[index];
    if (tree.leaf != undefined) {
      last.index = index;
      return copied;
    } else if (tree.node != undefined) {
      last.index = index;
      return copied.concat(getPathToFirst(tree));
    }

    index++;
  }

  copied.splice(-1);

  return findNext(copied);
}

function getAllegedLeaf(path: Path): Leaf | undefined {
  if (path.length == 0) {
    console.log("path.length == 0");
    return undefined;
  } else {
    const last = path[path.length - 1];
    return last.children[last.index].leaf as Leaf
  }
}

function ImageViewer({ selectedTree }: ImageViewerProps) {
  const [path, setPath] = React.useState<Path>(getPathToFirst(selectedTree));

  React.useEffect(
    () => setPath(getPathToFirst(selectedTree)),
    [selectedTree]
  );

  const leaf = getAllegedLeaf(path);
  const imgs = leaf != undefined ? getImg(leaf.filepath) : <></>;

  function onNext() {
    setPath(path => { return findNext(path); });
  }

  return (
    <>
      <Button onClick={onNext} variant="contained">Next</Button>
      <ImageList sx={{ height: 240 }} cols={3} rowHeight={164}>
        {imgs}
      </ImageList>
    </>
  );
}

function UIActive({ onCommit, tree }: AppProps) {
  const [selectedNode, setSelectedNode] = React.useState(getId(tree));
  const [treeForImageViewer, setTreeForImageViewer] = React.useState(tree);
  const [currentTreeView, setCurrentTreeView] = React.useState<MyTreeView>({ tree: tree, children: [] });

  function treeViewChangedCb(treeView: MyTreeView) {
    setCurrentTreeView(treeView);
  }

  function onCommitButtonClicked() {
    onCommit(currentTreeView);
  }

  const id2Tree = buildMap(tree);
  const selectedTree = id2Tree.get(selectedNode) as Tree;

  function onMoveEnd() {
    setTreeForImageViewer(selectedTree);
  }

  return (
    <Grid container spacing={2}>
      <Grid item xs={12}>
        <MapContainer center={position} zoom={13} scrollWheelZoom={true}>
          <TileLayer
            attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
            url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
          />
          <MyComponent onMoveEnd={onMoveEnd} selectedTree={selectedTree} treeView={currentTreeView} />
        </MapContainer>
      </Grid>

      <Grid item xs={6} className='grid'>
        <Button onClick={onCommitButtonClicked} variant="contained">Commit</Button>
        <MyTreeView tree={tree} iChangedAsTreeViewCb={treeViewChangedCb} onNodeSelect={id => setSelectedNode(id)}></MyTreeView>
      </Grid>

      <Grid item xs={6} className='grid'>
        <ImageViewer selectedTree={treeForImageViewer} ></ImageViewer>
      </Grid>
    </Grid>
  );
}
