pub const RESERVED_NAMES: &[&str] = &[
    "a16z",
    "a3capas",
    "aedile",
    "agryo",
    "aikin",
    "alexa",
    "amber",
    "ambergroup",
    "animal-guardians",
    "animalguardians",
    "anmi",
    "anmicapital",
    "anthonymq",
    "arkstream",
    "arkstreamcapital",
    "art",
    "aspect",
    "aspectventures",
    "astrome",
    "astrox",
    "atcapital",
    "aurora",
    "avatar",
    "aviate-labs",
    "aviatelabs",
    "avocado-research",
    "avocadoresearch",
    "axon",
    "b9-labs",
    "b9labs",
    "badge",
    "bartlett",
    "bauction",
    "beacon",
    "bebenture",
    "berkeley",
    "bigbuy",
    "bitcastle",
    "bitfinex",
    "bithumb",
    "bitkeep",
    "bittrex",
    "blockchain-heroes",
    "blockchainheroes",
    "blocks",
    "bluebird",
    "bob",
    "btc-flower",
    "btcflower",
    "bunchd",
    "bybit",
    "candid-convert",
    "candidconvert",
    "canister-tip-jar",
    "canistergeek",
    "canistertipjar",
    "canistore",
    "canlista",
    "cantorspace-teamup",
    "cantorspaceteamup",
    "cap",
    "catalyze",
    "cbd",
    "cetoswap",
    "chain-cloud",
    "chaincloud",
    "chainide",
    "checkrr",
    "cheersland",
    "cipherproxy",
    "civol",
    "cleverland",
    "codebase",
    "coin-flipper",
    "coinflipper",
    "content-fly",
    "contentfly",
    "cosmicrafts",
    "cover",
    "cramium",
    "cronic",
    "cronicnfts",
    "crowd-created-canvas",
    "crowdcreatedcanvas",
    "crowdeats",
    "crowdfund-nft",
    "crowdfundnft",
    "crowdrecords",
    "crowns",
    "crypto90s",
    "cryptoculturist",
    "cryptojian",
    "cryptospells",
    "cryptoworld",
    "cube-run",
    "cuberun",
    "cubic",
    "cycle",
    "cycle-dao",
    "cycledao",
    "cycles",
    "cycles-bet",
    "cyclesbet",
    "cyman",
    "dab",
    "dank",
    "dapp-man",
    "dappman",
    "data-app-development-platform",
    "dataappdevelopmentplatform",
    "databox",
    "dbox",
    "deckdeckgo",
    "deland",
    "demodroids",
    "departure-labs",
    "departurelabs",
    "devnull",
    "dfinance",
    "dfinity",
    "dfinity-academy-ru",
    "dfinity-alliance",
    "dfinity-bulls",
    "dfinity-club",
    "dfinity-community",
    "dfinity-donkeys",
    "dfinity-en-espanol",
    "dfinity-explorer",
    "dfinity-nuclear-sheep",
    "dfinity-scan",
    "dfinity-sz",
    "dfinity-vietnam",
    "dfinity-web-services",
    "dfinityacademyru",
    "dfinityalliance",
    "dfinitybulls",
    "dfinityclub",
    "dfinitycommunity",
    "dfinitydonkeys",
    "dfinityenespanol",
    "dfinityexplorer",
    "dfinitynuclearsheep",
    "dfinityscan",
    "dfinitysz",
    "dfinityvietnam",
    "dfinityvn",
    "dfinitywebservices",
    "dfistarter",
    "dflow",
    "dft",
    "dfusion",
    "dgdg",
    "difibase",
    "disto",
    "distrikt",
    "dlotto",
    "dmail",
    "dmail-network",
    "dmailnetwork",
    "documentingtheic",
    "dom",
    "doocoins",
    "dots",
    "dport",
    "drake",
    "draperdragon",
    "drip-land",
    "dripland",
    "dscvr",
    "dservice",
    "dsign",
    "dsocial",
    "dstar",
    "dwitter",
    "earth",
    "earth-wallet",
    "earthdao",
    "earthwallet",
    "economia",
    "eimolad",
    "elementum",
    "ensofinance",
    "entrepot",
    "epic-nfts-exponent",
    "epicnftsexponent",
    "eria",
    "evm-on-icp",
    "evmonicp",
    "ex3",
    "exponent",
    "fallingstars-nft",
    "fallingstarsnft",
    "fbg",
    "fbgcapital",
    "fenbushi",
    "fiona",
    "fleek",
    "fleek-co",
    "fleek-ooo",
    "fleekco",
    "fleekooo",
    "freedom",
    "freeos",
    "frog-nation",
    "frognation",
    "functionland",
    "gauzs",
    "get-impact-now",
    "getimpactnow",
    "gigaverse-labs",
    "gigaverselabs",
    "gobazzinga",
    "golang-developer-tools",
    "golangdevelopertools",
    "gpad",
    "hashkey",
    "hashkeycapital",
    "haundet-hamsters",
    "haundethamsters",
    "herbert",
    "hexgl",
    "holon",
    "huobi",
    "hyuna",
    "iaf",
    "ibridge",
    "ic",
    "ic-ape-ventures",
    "ic-birds",
    "ic-canvas",
    "ic-cards",
    "ic-dashboard",
    "ic-dinos",
    "ic-drip",
    "ic-drive",
    "ic-gallery",
    "ic-notes",
    "ic-rocks",
    "ic-tools-dart",
    "ic-turtles",
    "ic-vault",
    "ic-video",
    "ic-whiskers",
    "ic8",
    "ica-dashboard",
    "icadashboard",
    "icaliens",
    "icape-ventures",
    "icapeventures",
    "icapps",
    "icats",
    "icbirds",
    "icbunny",
    "iccanvas",
    "iccards",
    "iccomics",
    "icdashboard",
    "icdevs",
    "icdinos",
    "icdrip",
    "icdrive",
    "ices",
    "icevent",
    "icgallery",
    "icgiraffes",
    "icircle-nft",
    "icirclenft",
    "icity",
    "ickitsune",
    "ickitties",
    "ickoalas",
    "iclight",
    "iclighthouse",
    "icme",
    "icmoji",
    "icmoji-legends",
    "icmoji-origins",
    "icmojilegends",
    "icmojiorigins",
    "icnaming",
    "icnotes",
    "icnsid",
    "icp",
    "icp-art",
    "icp-birds",
    "icp-dog",
    "icp-france",
    "icp-global-fund",
    "icp-guide",
    "icp-league",
    "icp-neuron-calculator",
    "icp-squad",
    "icp-squad-nft",
    "icp-swap",
    "icp123",
    "icpad",
    "icpart",
    "icpbirds",
    "icpbunny",
    "icpcs",
    "icpdog",
    "icpets",
    "icpfans",
    "icpflower",
    "icpfrance",
    "icpglobalfund",
    "icpguide",
    "icphotographers",
    "icpics",
    "icpipeline",
    "icpis",
    "icpl",
    "icpl-community",
    "icplaces",
    "icplaunchpad",
    "icplcommunity",
    "icpleague",
    "icpmaps",
    "icpmeet",
    "icpneuroncalculator",
    "icport",
    "icprime8s",
    "icproject",
    "icproject-nft",
    "icprojectnft",
    "icpscan",
    "icpsquad",
    "icpsquadnft",
    "icpswap",
    "icpugs",
    "icpumpkin",
    "icpunks",
    "icpuppies",
    "icpverse",
    "icrocks",
    "icsnakes",
    "icspliffsters",
    "ictexas",
    "ictoolsdart",
    "icturtles",
    "icvault",
    "icvibesters",
    "icvideo",
    "icwallet",
    "icwave",
    "icwhiskers",
    "identity-labs",
    "identitylabs",
    "igf",
    "igor",
    "infernal-vampire-colony",
    "infernalvampirecolony",
    "infinite-chimp-nft-collection",
    "infinitechimpnftcollection",
    "infinity-swap",
    "infinityswap",
    "intelligent-nft-framework",
    "intelligentnftframework",
    "interconnect-network",
    "interconnectnetwork",
    "internet-astronauts",
    "internet-computer-based-blogs",
    "internet-computer-dashboard",
    "internet-computer-event-system",
    "internet-computer-wiki",
    "internet-humans",
    "internet-identity",
    "internetastronauts",
    "internetcomputer",
    "internetcomputerbasedblogs",
    "internetcomputerdashboard",
    "internetcomputereventsystem",
    "internetcomputerwiki",
    "internethumans",
    "internetidentity",
    "iosg",
    "irving",
    "isea",
    "itoka",
    "jan",
    "japan-dao-association",
    "japandaoassociation",
    "java-agent",
    "javaagent",
    "juicy-kicks",
    "juicykicks",
    "kawak",
    "klever",
    "kontribute",
    "kr1",
    "kryptonic",
    "kucoin",
    "learned",
    "light-wallet",
    "lightsail",
    "lightwallet",
    "liquid-icp",
    "liquidicp",
    "lo-fi-player",
    "lofiplayer",
    "lomesh",
    "marspool",
    "matoken",
    "meldd",
    "memecake",
    "messity",
    "metascore",
    "metasports",
    "metasports-basketball",
    "metasportsbasketball",
    "metaverse-ai",
    "metaverseai",
    "michaelhunte",
    "michaellee",
    "mishicat",
    "mission",
    "mission-is-possible",
    "missionispossible",
    "modclub",
    "modeclub",
    "motoko-day-drop",
    "motoko-library",
    "motoko-playground",
    "motoko-school",
    "motokodaydrop",
    "motokolibrary",
    "motokoplayground",
    "motokoschool",
    "mugatunes",
    "multcoin",
    "multicoincapital",
    "my-status",
    "mystatus",
    "near-future",
    "nearfuture",
    "nebulas",
    "neo",
    "newsie",
    "nfcity",
    "nft-english-auction",
    "nft-karts",
    "nft-studio",
    "nft-studio-marketplace",
    "nft-studio-minting-engine",
    "nft-village",
    "nftanvil",
    "nftenglishauction",
    "nftgaga",
    "nftkarts",
    "nftstudio",
    "nftstudiomarketplace",
    "nftstudiomintingengine",
    "nftvillage",
    "nftweb3",
    "nick",
    "nns",
    "nns-calculator",
    "nns-dapp",
    "nns-front-end-dapp",
    "nnscalculator",
    "nnsdao",
    "nnsdapp",
    "nnsfrontenddapp",
    "ns",
    "nuance",
    "og-medals",
    "ogmedals",
    "okex",
    "okx",
    "open-assessment-platform",
    "openassessmentplatform",
    "openchat",
    "optex",
    "orb",
    "orderswap",
    "origyn",
    "osmosis",
    "outliers",
    "outliersfund",
    "overchute",
    "paka",
    "parallel",
    "parallelventures",
    "party-board",
    "partyboard",
    "paul",
    "paulyoung",
    "perun-channels",
    "perunchannels",
    "photos",
    "plug",
    "pocket4d",
    "poked-bots",
    "pokedbots",
    "pokedstudio-bots",
    "pokedstudiobots",
    "polychain",
    "portal",
    "preangle",
    "prixelart",
    "prixers",
    "project-txa",
    "projecttxa",
    "psychedelic",
    "puzzle",
    "pythiania",
    "python-agent",
    "pythonagent",
    "qr-gallery",
    "qrgallery",
    "quark",
    "radius",
    "raisinrank",
    "ratels",
    "raver",
    "relation",
    "reversi",
    "rise-of-the-magni",
    "riseofthemagni",
    "rocklabs",
    "rust-libraries-for-ic-devs",
    "rustlibrariesforicdevs",
    "saga",
    "saga-tarot",
    "sagatarot",
    "sailfish",
    "sauveur",
    "savior",
    "scalar",
    "scalarcapital",
    "scinet",
    "secrets-of-the-midgaard-planet",
    "secretsofthemidgaardplanet",
    "shelf",
    "shine",
    "shinemine",
    "shobai-global",
    "shobaiglobal",
    "simdi",
    "simplifireic",
    "skydocs",
    "sling",
    "sly",
    "smart-contract-assets-within-the-metaverse",
    "smartcontractassetswithinthemetaverse",
    "smartpiggies",
    "sns",
    "snz",
    "sonic",
    "spare",
    "sparebytes",
    "spark",
    "sparkcapital",
    "sparkdigitalcapital",
    "speeqo",
    "springwind",
    "stoic-wallet",
    "stoicwallet",
    "sudograph",
    "supra-oracles",
    "supraoracles",
    "sushiswap-front-end",
    "sushiswapfrontend",
    "svangel",
    "synths",
    "tacen",
    "taggr",
    "team-bonsai",
    "teambonsai",
    "terabethia",
    "texas",
    "texas-poker",
    "texaspoker",
    "the-ic-gallery",
    "the-island-collective",
    "the-nft-village",
    "the-sword-nft",
    "the-video-canister",
    "the-wall",
    "theicgallery",
    "theislandcollective",
    "thenftvillage",
    "thespiderdao",
    "theswordnft",
    "thevideocanister",
    "thewall",
    "threearrows",
    "thuba",
    "tingram",
    "toniq",
    "toniq-labs",
    "toniqlabs",
    "triip",
    "uniswap",
    "uniswap-front-end-on-the-ic",
    "uniswapfrontendontheic",
    "unite",
    "unity",
    "unity3d",
    "unreal",
    "usergeek",
    "verific",
    "videate",
    "viewar-by-wearfits",
    "viewarbywearfits",
    "villageglobal",
    "vincrypto",
    "waterslide",
    "weact",
    "weave",
    "weavers",
    "web3r-chat",
    "web3rchat",
    "webi-ai",
    "webiai",
    "week-in-dfinity-news",
    "weekindfinity",
    "weekindfinitynews",
    "welcome-into-the-metaverse",
    "welcomeintothemetaverse",
    "wicp",
    "wild-and-west",
    "wildandwest",
    "wind",
    "wrapped-trillion-cycles",
    "wrappedtrillioncycles",
    "xr-foundation",
    "xrfoundation",
    "xtc",
    "ygg",
    "yolo-club",
    "yoloclub",
    "young",
];
