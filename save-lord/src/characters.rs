use serde::{Serialize, Deserialize};
use crate::skills::{Skill, SkillTarget, SkillEffect};
use crate::types::{Position, UnitType};

/// A character template (before being instantiated in a team).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterTemplate {
    pub id: u32,
    pub name: String,
    pub unit_type: UnitType,
    pub base_atk: i32,
    pub base_hp: i32,
    pub base_spd: i32,
    pub preferred_position: Position,
    pub normal_attack: Skill,
    pub move_skill: Skill,
    pub sp_skill: Option<Skill>,
    pub cd_skill: Option<Skill>,
    pub energy_skill: Option<Skill>,
    pub ultimate: Option<Skill>,
}

/// An instantiated character in a team/battle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub template: CharacterTemplate,
    pub star_level: u32,
    pub position: Position,
    pub hp: i32,
    pub max_hp: i32,
    pub atk: i32,
    pub def: i32,
    pub spd: i32,
    pub active_points: u32,
    pub passive_points: u32,
    pub max_active_points: u32,
    pub max_passive_points: u32,
    pub energy: i32,
    pub cd_remaining: i32,
    pub ult_used: bool,
    pub is_lord: bool,
    pub is_dead: bool,
    pub equipment: Vec<String>,
    pub buffs: Vec<crate::types::Buff>,
    pub shield: i32,
    pub revived_once: bool,
}

impl Character {
    pub fn from_template(tmpl: CharacterTemplate) -> Self {
        Self {
            hp: tmpl.base_hp,
            max_hp: tmpl.base_hp,
            atk: tmpl.base_atk,
            def: 0,
            spd: tmpl.base_spd,
            star_level: 1,
            position: tmpl.preferred_position,
            active_points: 3,
            passive_points: 2,
            max_active_points: 3,
            max_passive_points: 2,
            energy: 0,
            cd_remaining: 0,
            ult_used: false,
            is_lord: false,
            is_dead: false,
            equipment: Vec::new(),
            buffs: Vec::new(),
            shield: 0,
            revived_once: false,
            template: tmpl,
        }
    }

    pub fn effective_atk(&self) -> f64 {
        let mut atk = self.atk as f64;
        for b in &self.buffs { atk *= 1.0 + b.atk_mod; }
        atk
    }
    pub fn effective_def(&self) -> f64 {
        let mut def = self.def as f64;
        for b in &self.buffs { def *= 1.0 + b.def_mod; }
        def
    }
    pub fn effective_spd(&self) -> f64 {
        let mut spd = self.spd as f64;
        for b in &self.buffs { spd *= 1.0 + b.spd_mod; }
        spd
    }

    pub fn take_damage(&mut self, dmg: i32) -> i32 {
        let mut remaining = dmg;
        if self.shield > 0 {
            let absorbed = self.shield.min(remaining);
            self.shield -= absorbed;
            remaining -= absorbed;
        }
        self.hp -= remaining;
        if self.hp <= 0 { self.hp = 0; self.is_dead = true; }
        remaining
    }

    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.max_hp);
        if self.hp > 0 { self.is_dead = false; }
    }

    pub fn name(&self) -> &str { &self.template.name }
}

// ---- Procedural name generation and 1000 character creation ----

const FIRST_NAMES: &[&str] = &[
    "Aaron","Abigail","Adam","Adrian","Aelric","Agnes","Alan","Alaric","Albert","Aldric",
    "Alexander","Alfred","Alice","Alistair","Althea","Amara","Amelia","Anders","Andrea","Andrew",
    "Angelica","Anna","Anthony","Archer","Aria","Ariana","Arin","Arthur","Astor","Athena",
    "Audrey","Aurora","Axel","Baldric","Balthazar","Barbara","Bard","Bartholomew","Beatrice","Beatrix",
    "Belinda","Benedict","Benjamin","Bernard","Bertrand","Bethany","Bianca","Bjorn","Blair","Blake",
    "Borak","Boris","Branwen","Brianna","Bruce","Brunhild","Bryce","Cade","Caedmon","Cain",
    "Caleb","Calista","Callum","Camilla","Caspian","Cassandra","Cassius","Cedric","Celeste","Celia",
    "Charles","Charlotte","Chester","Chloe","Christian","Christina","Cian","Claire","Clarence","Clarissa",
    "Claudius","Clayton","Clement","Clifford","Clint","Clovis","Cole","Colin","Conrad","Cora",
    "Corbin","Corinna","Cornelius","Crispin","Cynthia","Cyrus","Dagon","Dahlia","Damian","Damien",
    "Damon","Daniel","Daphne","Darius","Darlene","David","Deirdre","Delia","Derek","Desmond",
    "Diana","Diego","Dieter","Dominic","Donald","Donovan","Dorian","Dorothy","Douglas","Drake",
    "Dreyfus","Duncan","Dustin","Dylan","Eadric","Eamon","Earl","Edgar","Edith","Edmund",
    "Edward","Edwin","Effie","Eileen","Elaine","Eleanor","Elena","Elias","Elijah","Eliza",
    "Elizabeth","Ella","Elric","Elyan","Emeline","Emery","Emilia","Emily","Emma","Emmett",
    "Eric","Erin","Ernest","Eros","Esme","Ethan","Ethel","Eugene","Eva","Evelyn",
    "Everett","Ewan","Fabian","Faye","Felicia","Felix","Ferdinand","Fergus","Finn","Fiona",
    "Flavia","Fletcher","Flora","Florian","Flynn","Frances","Francis","Frederick","Freya","Gabriel",
    "Gareth","Garrett","Garrick","Gawain","Genevieve","Geoffrey","George","Gerald","Gerard","Gideon",
    "Gilbert","Giles","Giselle","Gloria","Goddard","Gordon","Grace","Grant","Gregory","Gretchen",
    "Griffin","Guinevere","Gunther","Gwendolyn","Hadrian","Hailey","Hal","Hale","Hamilton","Hamish",
    "Hannah","Harold","Harriet","Harrison","Harry","Hattie","Hector","Helen","Helena","Helios",
    "Helmut","Henrietta","Henry","Herbert","Herman","Hilda","Hildegard","Hiram","Holden","Holly",
    "Horace","Howard","Hubert","Hudson","Hugh","Hugo","Humphrey","Hunter","Ian","Igraine",
    "Ignatius","Igor","Ilya","Imelda","Imogen","Inara","Ingrid","Irene","Iris","Irving",
    "Isaac","Isabella","Isolde","Ivan","Ivor","Ivy","Jack","Jacob","Jade","James",
    "Jamie","Janus","Jared","Jasmine","Jason","Jasper","Jean","Jenna","Jeremy","Jerome",
    "Jessica","Jocelyn","Johan","John","Jonah","Jordan","Josephine","Joshua","Joyce","Judith",
    "Julian","Juliana","Julius","Juniper","Justina","Justin","Kael","Kara","Karen","Karl",
    "Kassandra","Katarina","Keegan","Keith","Kellan","Kelly","Kendrick","Kenneth","Kent","Kestrel",
    "Kevin","Kiera","Killian","Kimberly","Kira","Klaus","Korbin","Kurt","Kyle","Lancelot",
    "Lara","Larissa","Laura","Laurel","Lawrence","Leander","Lee","Lena","Leo","Leon",
    "Leonard","Leopold","Leslie","Lester","Levi","Liam","Lila","Lilith","Lily","Lincoln",
    "Linden","Lionel","Livia","Llewellyn","Lloyd","Logan","Lola","Lorenzo","Loretta","Lorraine",
    "Lothar","Louis","Louisa","Lucas","Lucian","Lucilla","Lucius","Lucy","Ludwig","Luke",
    "Luther","Lydia","Lyra","Lysander","Mabel","Madeline","Magnus","Malcolm","Mara","Marcus",
    "Margaret","Maria","Marianne","Marina","Marion","Mark","Marlene","Marshall","Martha","Martin",
    "Marvin","Mary","Matthew","Matthias","Maud","Maureen","Maurice","Maximilian","May","Maya",
    "Meghan","Melanie","Melchior","Melissa","Mercy","Meredith","Merlin","Mia","Michael","Michelle",
    "Miles","Millicent","Milton","Minerva","Mirabel","Miranda","Miroslav","Mitchell","Molly","Morgan",
    "Mortimer","Muriel","Nadia","Nancy","Naomi","Natalia","Nathan","Nathaniel","Ned","Neil",
    "Nelson","Nero","Nessa","Nestor","Neville","Niall","Nicholas","Nigel","Nina","Noah",
    "Noel","Nolan","Nora","Norman","Norbert","Nova","Octavia","Octavius","Odette","Odysseus",
    "Olaf","Oliver","Olivia","Ophelia","Orlando","Osric","Oswald","Otis","Otto","Owen",
    "Pandora","Patricia","Patrick","Paul","Paula","Percival","Percy","Peter","Petra","Philip",
    "Phillipa","Phoebe","Phineas","Pierce","Piers","Pietro","Polly","Portia","Priscilla","Quentin",
    "Quincy","Quinn","Rachel","Rafael","Raina","Ralph","Randall","Randolph","Raphael","Raquel",
    "Raven","Raymond","Rebecca","Reginald","Reinhardt","Remus","Renata","Renee","Reuben","Rex",
    "Rhonda","Rhys","Richard","Rita","Robert","Robin","Roderick","Rodney","Roger","Roland",
    "Rolf","Roman","Ronald","Rosalind","Rosamund","Roscoe","Rose","Rowan","Roy","Rudolph",
    "Rufus","Rupert","Russell","Ruth","Ryan","Sabrina","Sadie","Salem","Samson","Samuel",
    "Sandra","Sara","Sarah","Sebastian","Selena","Seraphina","Serena","Seth","Seymour","Shane",
    "Sharon","Sheila","Sheridan","Sibyl","Sidney","Sigrid","Silas","Silvana","Simon","Soren",
    "Spencer","Stanley","Stella","Stephen","Stuart","Sullivan","Susanna","Sven","Sylvia","Tabitha",
    "Talia","Tamsin","Tara","Tatiana","Terence","Teresa","Thaddeus","Thea","Theodore","Theresa",
    "Thomas","Thor","Tiberius","Tobias","Tristan","Trudy","Ulric","Ulysses","Una","Ursula",
    "Valentina","Valerius","Vanessa","Varian","Vera","Vernon","Veronica","Victor","Victoria","Vincent",
    "Viola","Violet","Virgil","Vivian","Vladimir","Vivienne","Waldo","Walter","Wanda","Warren",
    "Wayne","Wendell","Wendy","Werner","Wilbur","Wilfred","Wilhelm","Willa","William","Winifred",
    "Winston","Wolfgang","Xander","Xavier","Xenia","Yara","Yasmine","Yorick","Yvonne","Zachary",
    "Zara","Zebulon","Zelda","Zephyr","Zoe"
];

const LAST_NAMES: &[&str] = &[
    "Ashford","Ashworth","Atwood","Auburn","Austen","Barclay","Barlow","Barnett","Barrett","Barton",
    "Beaumont","Beckett","Bellamy","Bennett","Benson","Bentley","Bertrand","Black","Blackburn","Blackwood",
    "Blair","Blake","Bloomfield","Boleyn","Bolton","Booker","Boone","Booth","Bosworth","Bouchard",
    "Bradford","Bradley","Brandt","Branson","Braxton","Brennan","Brewster","Briar","Bridgewater","Bright",
    "Brock","Brody","Brooks","Broughton","Brown","Bruce","Brunswick","Bryant","Buchanan","Buckley",
    "Buckner","Bullard","Burgess","Burke","Burnett","Burnham","Burr","Burton","Butler","Byrne",
    "Cabot","Cade","Caldwell","Calhoun","Callahan","Cameron","Campbell","Canfield","Cantrell","Carew",
    "Carleton","Carlyle","Carrington","Carroll","Carson","Carter","Cartwright","Carver","Cary","Cash",
    "Cassidy","Castellan","Catesby","Chadwick","Chalmers","Chamberlain","Chandler","Channing","Chapman","Chase",
    "Chatterton","Chauncey","Cheney","Chesterfield","Childers","Chilton","Chisholm","Christie","Churchill","Clancy",
    "Claremont","Clarence","Clarke","Clayton","Cleave","Clement","Clermont","Cleveland","Clifford","Clifton",
    "Clinton","Clive","Close","Clyde","Coburn","Cochran","Coffey","Colbert","Colburn","Cole",
    "Coleridge","Collier","Colton","Colver","Compton","Conant","Congreve","Conley","Connolly","Connor",
    "Conrad","Conroy","Constance","Conti","Conway","Cooke","Cooper","Copeland","Corbin","Corcoran",
    "Corey","Cornell","Cornwall","Corwin","Cotterell","Cotton","Courtney","Coventry","Covington","Cowan",
    "Cowper","Cox","Crabtree","Craft","Craig","Crane","Cranfield","Cranston","Crawford","Creighton",
    "Crescent","Crichton","Croft","Cromwell","Cronin","Crook","Crosby","Cross","Crowley","Crozier",
    "Cruickshank","Culpepper","Cumberland","Cummings","Cunningham","Currier","Curtis","Cushing","Custer","Dale",
    "Dalton","Daly","Dane","Danvers","Darby","Darcy","Darnley","Davenport","Davidson","Davies",
    "Davis","Dawson","Day","Dayton","Dean","Deane","Decker","Delacroix","Delaney","Delany",
    "Delmar","Dempsey","Denham","Denholm","Dennehy","Denton","Denver","Derby","Dering","Devereux",
    "Devlin","DeWitt","Dexter","Dickens","Dickerson","Dickinson","Digby","Dillinger","Dillon","Disney",
    "Disraeli","Dobbs","Dobson","Dodd","Dodge","Dolby","Donahue","Donnelly","Donovan","Dooley",
    "Dorchester","Doremus","Dorian","Dorset","Doty","Doubleday","Doughty","Douglas","Dow","Doyle",
    "Drake","Draper","Drayton","Drew","Drummond","Drury","Dryden","DuBois","Dudley","Duff",
    "Duffy","Duke","Dumond","Dunaway","Dunbar","Duncan","Dunham","Dunlop","Dunmore","Dunn",
    "Dunne","Durand","Durant","Durham","Dustin","Dutton","Duval","Dwight","Dwyer","Dyce",
    "Dyer","Dykstra","Eads","Earl","Eastman","Easton","Eaton","Ebert","Eckhart","Eddington",
    "Eden","Edgerton","Edison","Edmonds","Edmunds","Edwards","Egan","Eglinton","Egmont","Ehrlich",
    "Eisenhower","Elgin","Eliot","Ellery","Ellington","Elliot","Ellis","Ellison","Ellsworth","Elms",
    "Elroy","Elsdon","Elton","Ely","Embry","Emerson","Emery","Emmett","Endicott","Engel",
    "English","Enright","Epperson","Erickson","Erwin","Essex","Estes","Esterbrook","Estes","Ethelbert",
    "Eustace","Evans","Evatt","Everest","Everett","Evers","Ewell","Ewing","Exeter","Fabian",
    "Fairbanks","Fairchild","Fairfax","Fairweather","Falconer","Falkner","Fane","Farley","Farmer","Farnham",
    "Farnsworth","Farquhar","Farr","Farragut","Farrar","Farrell","Faulkner","Fawcett","Fellows","Felton",
    "Fenwick","Ferdinand","Ferguson","Ferrers","Ferris","Field","Fielding","Finch","Finlay","Finn",
    "Firth","Fisher","Fisk","Fitch","Fitzgerald","Fitzgibbon","Fitzpatrick","Fitzroy","Flagg","Flanders",
    "Flannagan","Flannery","Flavel","Fleet","Fleetwood","Fleming","Fletcher","Flint","Flint","Flood",
    "Flora","Florian","Flowers","Floyd","Flynn","Fogarty","Foley","Folger","Follett","Forbes",
    "Ford","Forester","Forrest","Forster","Forsyth","Fortescue","Fosdick","Foss","Foster","Fountain",
    "Fowkes","Fowler","Fox","Foxe","Foyle","Frame","Francis","Franklin","Fraser","Frawley",
    "Frazer","Frederick","Freedman","Freeman","Freemont","Fremont","French","Frick","Friedman","Frost",
    "Fry","Fuller","Fulton","Furness","Gable","Gadsden","Gage","Gaither","Gale","Gallagher",
    "Galloway","Gallup","Galvin","Gamble","Gannett","Garber","Gardiner","Gardner","Garfield","Garland",
    "Garner","Garrick","Garrison","Garth","Gaskill","Gates","Gatlin","Gavin","Gaylord","Geary",
    "Geddes","Gentry","George","Gerald","Germaine","Gerrish","Gershwin","Getty","Ghent","Gibbon",
    "Gibbons","Gibbs","Gibson","Giddings","Gifford","Gilbert","Gilchrist","Giles","Gill","Gillespie",
    "Gillette","Gilliam","Gilligan","Gilman","Gilmer","Gilmore","Girard","Givens","Gladden","Gladstone",
    "Glasgow","Glass","Glazier","Gleason","Glenn","Glenville","Glover","Goddard","Godfrey","Godwin",
    "Goethe","Goff","Golden","Golding","Goldman","Goldsmith","Goldstein","Gonzalez","Goodall","Goode",
    "Goodfellow","Goodhue","Goodman","Goodrich","Goodwin","Goodyear","Gordon","Gore","Gorman","Gorsuch",
    "Goshen","Goss","Gothic","Gough","Gould","Grafton","Graham","Grainger","Granger","Grant",
    "Grantham","Granville","Gratton","Graves","Gray","Grayson","Green","Greenberg","Greene","Greenfield",
    "Greenhill","Greenleaf","Greenough","Greer","Gregory","Grenfell","Grenville","Gresham","Grey","Gridley",
    "Grier","Grieve","Griffin","Griffith","Griffiths","Grim","Grimes","Grimke","Griswold","Grosvenor",
    "Grover","Grubb","Grundy","Guernsey","Guild","Guinness","Gulliver","Gunn","Gunter","Gunther",
    "Gurney","Guthrie","Guy","Gwinn","Gwynne","Haas","Hackett","Haddon","Hadfield","Hadley",
    "Haffner","Hagan","Haggard","Hahn","Haig","Haines","Halberstam","Haldane","Hale","Haley",
    "Halifax","Hall","Hallowell","Halpern","Halsey","Halsted","Hambly","Hamilton","Hamlin","Hammond",
    "Hampden","Hampton","Hancock","Hand","Handy","Haney","Hankin","Hanks","Hanley","Hanna",
    "Hanscom","Hansen","Hanson","Harbin","Harbinger","Harcourt","Harden","Hardie","Hardin","Harding",
    "Hardwick","Hardy","Hargrave","Hargrove","Harkness","Harlan","Harland","Harlech","Harley","Harlow",
    "Harmon","Harnett","Harper","Harrell","Harrington","Harris","Harrison","Harrod","Hart","Harte",
    "Hartford","Hartigan","Hartley","Hartmann","Hartwell","Harvey","Haskell","Haskins","Hastings","Hatch",
    "Hathaway","Hathorne","Hatton","Haverhill","Haviland","Hawke","Hawkins","Hawley","Hawthorne","Hay",
    "Hayden","Hayes","Haynes","Hays","Hayward","Haywood","Hazard","Healy","Hearst","Heath",
    "Heather","Heaton","Heflin","Heinz","Heller","Helms","Helmsley","Hemingway","Hemphill","Henderson",
    "Hendricks","Hendrix","Henley","Henry","Henshaw","Hepburn","Herbert","Herndon","Herrick","Hershey",
    "Herzog","Hess","Hester","Hetherington","Hewes","Hewitt","Heyward","Hibbard","Hickman","Hickok",
    "Hicks","Higginbotham","Higgins","Hildebrand","Hill","Hillary","Hillman","Hills","Hilton","Hinchcliffe",
    "Hinckley","Hinde","Hindley","Hinds","Hinkle","Hinton","Hitchcock","Hoadley","Hoar","Hobart",
    "Hobbes","Hobbs","Hobhouse","Hobsbaum","Hockley","Hodges","Hodgkin","Hoffman","Hogan","Holbrook",
    "Holcombe","Holden","Holder","Holgate","Holiday","Holland","Holliday","Hollingsworth","Holloway","Holm",
    "Holman","Holmes","Holt","Holton","Holyoke","Home","Homer","Hone","Hood","Hook",
    "Hooker","Hooper","Hope","Hopkins","Hopwood","Horan","Horder","Horn","Horne","Horsley",
    "Horton","Hoskins","Hotchkiss","Hough","House","Houston","Hovenden","Hovey","Howard","Howe",
    "Howell","Howland","Hoyt","Hubbard","Hubbell","Huber","Hudson","Huff","Hughes","Hughitt",
    "Hulbert","Hull","Humbert","Hume","Hummel","Humphrey","Humphreys","Hunnewell","Hunt","Hunter",
    "Huntington","Huntley","Hurd","Hurlbut","Hurley","Hurst","Hussey","Husted","Hutchins","Hutchinson",
    "Huxley","Hyde","Hylan","Hylton","Hyman","Ibbotson","Ibsen","Ickes","Iglehart","Ingalls",
    "Ingersoll","Ingham","Ingle","Inglis","Ingram","Inman","Innes","Ireland","Irons","Irvine",
    "Irving","Irwin","Isaacs","Isham","Ives","Ivey","Ivins","Izard","Jackman","Jackson",
    "Jacobs","Jagger","James","Jameson","Janeway","Janney","Jaques","Jarman","Jarratt","Jarrett",
    "Jarvis","Jasper","Jastrow","Jay","Jeffers","Jefferson","Jeffery","Jeffries","Jekyll","Jenkins",
    "Jenner","Jennings","Jensen","Jepson","Jernigan","Jerome","Jervis","Jessup","Jewel","Jewell",
    "Jewett","Jocelyn","Johns","Johnson","Johnston","Jolliffe","Jonas","Jones","Jonker","Jopling",
    "Joplin","Jordan","Jorgensen","Joseph","Jouett","Jowett","Joy","Joyce","Joyner","Judd",
    "Judge","Judson","Jukes","Julian","June","Jung","Juniper","Junkin","Jury","Kane",
    "Kavanaugh","Kay","Kaye","Kean","Keane","Kearney","Kearns","Keating","Keats","Keble",
    "Keene","Keffer","Keith","Kellar","Keller","Kelley","Kellogg","Kelly","Kelsey","Kemble",
    "Kempe","Kendall","Kendrick","Kenilworth","Kennan","Kennedy","Kenner","Kennett","Kenny","Kent",
    "Kenyon","Keogh","Kepler","Ker","Kern","Kerr","Kerry","Kessler","Ketchum","Key",
    "Keyes","Keys","Kidd","Kidder","Kiefer","Kilbourn","Kilbride","Kilgore","Kilmer","Kilpatrick",
    "Kimball","Kincaid","King","Kingdon","Kingsley","Kingston","Kinnaird","Kinney","Kinsolving","Kipling",
    "Kirby","Kirk","Kirkland","Kirkpatrick","Kirkwood","Kitchell","Kittredge","Klein","Kline","Knapp",
    "Knight","Knights","Knowles","Knox","Koch","Koehler","Kohl","Kohler","Kohn","Kolb",
    "Koontz","Kramer","Krebs","Krueger","Kuhn","Kurtz","Kyle","Lacey","Lackland","Lacy",
    "Ladd","Lafayette","Lafferty","Laird","Lake","Lamar","Lamb","Lambert","Lampton","Lancaster",
    "Landon","Landry","Lane","Lang","Langdon","Langford","Langhorne","Langley","Langston","Lanier",
    "Lankford","Lanning","Lansing","Lapham","Laramore","Larkin","Larned","Larrabee","Larsen","Larson",
    "Latham","Lathrop","Latimer","Latta","Lauder","Lauderdale","Laughlin","Laurens","Laurie","Law",
    "Lawford","Lawler","Lawless","Lawrence","Lawson","Lawton","Lay","Layton","Lea","Leach",
    "Leacock","Leary","Leavitt","Lechmere","Ledbetter","Ledyard","Lee","Leeds","Leek","Leete",
    "Lefevre","Lefferts","Leffingwell","Legare","Lehman","Leibowitz","Leigh","Leighton","Leitch","Leland",
    "Lemieux","Lennox","Lent","Leo","Leonard","Leopold","Lerner","Leroy","Leslie","Lester",
    "Levine","Levy","Lewes","Lewis","Ley","Libbey","Light","Lightfoot","Lilienthal","Lillard",
    "Lilly","Lim","Lincoln","Lind","Lindbergh","Lindley","Lindsay","Lindsey","Linton","Lippincott",
    "Lippitt","Lipton","Litchfield","Littauer","Little","Littlefield","Livingston","Livsey","Llewellyn","Lloyd",
    "Locke","Lockhart","Lockwood","Lodge","Loeb","Logan","Lomax","Lombard","London","Long",
    "Longfellow","Longstreet","Longworth","Loomis","Lord","Lorimer","Loring","Lothrop","Lott","Loughead",
    "Lounsbury","Love","Lovejoy","Lovell","Lovering","Low","Lowden","Lowell","Lowery","Lowndes",
    "Lubbock","Lucas","Luce","Luckey","Ludington","Ludlow","Lugar","Lukens","Lumpkin","Lund",
    "Lunt","Lupo","Lusk","Lutes","Luther","Lutyens","Luxford","Lyford","Lyle","Lyman",
    "Lynch","Lynde","Lyndon","Lynn","Lyon","Lyons","Lytle","Lytton","MacArthur","Macauley",
    "MacDonald","MacDougall","MacFarlane","MacGregor","MacIntyre","MacKay","MacKenzie","MacLaren","MacLean","MacLeod",
    "MacMahon","MacNeil","MacPherson","Macy","Maddox","Madigan","Madison","Magill","Magnuson","Magruder",
    "Mahan","Mahoney","Maitland","Major","Mallory","Malloy","Malone","Maloney","Maltby","Manchester",
    "Mandel","Manfred","Mangum","Manley","Mann","Manning","Mansfield","Manship","Manson","Manville",
    "Mapes","Marble","March","Marchand","Marcy","Marion","Mark","Markham","Marks","Marland",
    "Marlow","Marlowe","Marple","Marquand","Marquis","Marriott","Marsh","Marshall","Marston","Martin",
    "Martindale","Martinez","Marvin","Mason","Massie","Masterson","Mather","Mathews","Mathis","Matson",
    "Matteson","Matthew","Matthews","Matthias","Mattingly","Mauldin","Mauro","Maury","Maxcy","Maxey",
    "Maxwell","May","Mayberry","Mayer","Mayfield","Maynard","Mayo","McAdoo","McAllister","McBride",
    "McCall","McCallum","McCandless","McCann","McCarran","McCarthy","McCarty","McCauley","McClellan","McClintock",
    "McCloskey","McCloud","McClure","McClurg","McCollum","McComb","McConaughey","McConnell","McCord","McCormack",
    "McCormick","McCoy","McCrae","McCray","McCrea","McCreary","McCullagh","McCullers","McCulloch","McCullough",
    "McCutcheon","McDaniel","McDermott","McDevitt","McDonald","McDougall","McDowell","McDuffie","McElroy","McEwen",
    "McFarland","McGee","McGill","McGinley","McGinnis","McGovern","McGowan","McGrath","McGraw","McGregor",
    "McGuire","McHenry","McHugh","McInerney","McIntosh","McIntyre","McKay","McKean","McKee","McKellar",
    "McKelvey","McKenna","McKenzie","McKibbin","McKinley","McKinney","McKinnon","McKittrick","McLane","McLaren",
    "McLaughlin","McLean","McLennan","McLeod","McLoughlin","McMahon","McManus","McMaster","McMillan","McMillen",
    "McMullen","McMurray","McNair","McNally","McNamara","McNaughton","McNeal","McNeil","McNeill","McNutt",
    "McPhee","McPherson","McQuaid","McQueen","McRae","McShane","McWilliams","Meade","Meadows","Meagher",
    "Medary","Meek","Meeker","Megrue","Meikle","Melcher","Mellen","Melton","Melville","Mencken",
    "Mendenhall","Mercer","Meredith","Meriwether","Merrick","Merrill","Merritt","Merwin","Messer","Metcalf",
    "Mettler","Meyer","Meyers","Michaels","Michener","Mickey","Middleton","Midgley","Mifflin","Miles",
    "Milford","Millard","Miller","Millikan","Milliken","Mills","Milne","Milner","Milton","Miner",
    "Minot","Minter","Miranda","Mish","Mitchell","Mitchill","Mixer","Mizner","Moffat","Moffett",
    "Moffitt","Moir","Molesworth","Molineux","Monaghan","Monckton","Moncrieff","Monroe","Monsarrat","Montague",
    "Montgomery","Moody","Mooney","Moore","Moos","Moran","Morehead","Morehouse","Moreland","Moreno",
    "Morgan","Morley","Morrell","Morrill","Morris","Morrison","Morrissey","Morrow","Morse","Mortimer",
    "Morton","Moseley","Moses","Mosher","Mosley","Moss","Mott","Moulton","Mount","Mowbray",
    "Mowry","Moyers","Moynihan","Muir","Muldoon","Mulholland","Mullan","Muller","Mulligan","Mullins",
    "Mulock","Mumford","Munford","Munroe","Munson","Murchison","Murdoch","Murdock","Murfree","Murphey",
    "Murphy","Murray","Murrow","Murtagh","Musgrave","Musser","Musto","Myer","Myers","Mygatt",
    "Myrick","Nabors","Nagle","Nash","Nason","Nathan","Nay","Neal","Nearing","Neff",
    "Neighbor","Neil","Neill","Neilson","Nelson","Nesbitt","Nesmith","Nettleton","Neuberger","Nevins",
    "Newbegin","Newberry","Newbery","Newbold","Newcastle","Newcomb","Newcomer","Newell","Newhall","Newkirk",
    "Newman","Newmark","Newsom","Newton","Niblack","Nicholas","Nichols","Nicholson","Nickerson","Nicol",
    "Nicolls","Niles","Nisbet","Niven","Nixon","Noble","Noel","Noggle","Nolan","Norcross",
    "Nordhoff","Norfolk","Norman","Norris","North","Northcott","Northrop","Northrup","Norton","Norvell",
    "Norwood","Nott","Nourse","Noyes","Nuckols","Nugent","Null","Nunn","Nye","Oakes",
    "Oakley","Oates","Ober","Oberholtzer","Oberlin","Oboler","O'Brien","Ochs","O'Connell","O'Connor",
    "Odell","Odiorne","O'Donnell","Oenslager","Offutt","Ogden","Ogilvie","Ogle","Oglethorpe","O'Hara",
    "O'Keefe","Oldfield","Olds","O'Leary","Olin","Oliver","Olivier","Olmsted","Olney","Olsen",
    "Olson","O'Malley","O'Neil","O'Neill","Opdyke","Oppenheimer","Ordway","O'Reilly","Ormsby","Orr",
    "Orrick","Orton","Orville","Osborn","Osborne","Osbourne","Osgood","O'Shaughnessy","O'Shea","Osler",
    "Ostrander","Oswald","Otis","O'Toole","Ott","Ottinger","Oursler","Overacker","Overbey","Overbury",
    "Overman","Oviatt","Owen","Owens","Oxnard","Pabst","Pace","Pack","Packard","Padelford",
    "Padgett","Page","Paget","Paine","Painter","Palfrey","Palmer","Palmore","Pangborn","Pardee",
    "Parent","Paris","Parish","Park","Parker","Parkes","Parkhurst","Parkinson","Parkman","Parks",
    "Parmelee","Parnell","Parr","Parrish","Parrott","Parry","Parsons","Partridge","Paschall","Patch",
    "Patchen","Paterson","Paton","Patrick","Patten","Patterson","Patton","Paul","Paulding","Paxson",
    "Paxton","Payne","Peabody","Peach","Peacock","Pearce","Pearl","Pearsall","Pearse","Pearson",
    "Pease","Peck","Peckham","Pedersen","Peebles","Peet","Pell","Pelletier","Pemberton","Pendleton",
    "Penfield","Penn","Penney","Pennington","Pennypacker","Penrose","Percy","Perham","Perkins","Perley",
    "Perry","Pershing","Person","Peter","Peters","Peterson","Pettibone","Pettigrew","Pettis","Pettit",
    "Pew","Peyton","Pfeiffer","Phelan","Phelps","Pharr","Phetteplace","Philbrick","Philips","Phillips",
    "Phillpotts","Philpott","Phinney","Physick","Piatt","Pickens","Pickering","Pickett","Pierce","Pierson",
    "Pike","Pillsbury","Pinchot","Pinkerton","Pinkham","Pinney","Piper","Pitcairn","Pitcher","Pitkin",
    "Pitman","Pitt","Pittenger","Pitts","Place","Plank","Platt","Playfair","Pleasonton","Plimpton",
    "Plumb","Plumer","Plunkett","Poage","Poe","Poffenberger","Polk","Pollard","Pollett","Pollitt",
    "Pollock","Pomerene","Pomeroy","Pool","Poole","Pope","Porcher","Porter","Portland","Posey",
    "Post","Poston","Potter","Potts","Poucher","Pound","Powderly","Powell","Power","Powers",
    "Poyntz","Pratt","Prentiss","Presbrey","Prescott","Preston","Price","Prichard","Pride","Priest",
    "Prime","Prince","Pringle","Prioleau","Pritchard","Procter","Proctor","Prouty","Pruyn","Pryor",
    "Puckett","Pugh","Pulitzer","Pullman","Pumphrey","Purdy","Putnam","Pye","Pyle","Pynchon",
    "Quain","Quarles","Quay","Queeny","Quigg","Quimby","Quincy","Quinn","Quint","Quirk",
    "Rabi","Rabinowitz","Raby","Rachal","Racine","Radcliffe","Rader","Radford","Ragan","Raines",
    "Rains","Rainey","Rainsford","Raleigh","Ralston","Ramey","Ramsay","Ramsdell","Ramsey","Rand",
    "Randall","Randolph","Raney","Range","Ranger","Rankin","Ransom","Rantoul","Raper","Rapp",
    "Rascoe","Rash","Raskob","Rasmussen","Rathbone","Rathbun","Ratliff","Rauch","Rauschenbusch","Rawlings",
    "Rawlins","Rawlinson","Rawson","Ray","Rayburn","Raye","Raymond","Raynor","Rea","Read",
    "Reade","Reading","Reagan","Reavis","Redfield","Redmond","Reece","Reed","Rees","Reese",
    "Reeve","Reeves","Regan","Reid","Reilly","Rein","Reinhardt","Reis","Reisinger","Remick",
    "Renick","Reno","Rentschler","Replogle","Requa","Resor","Revelle","Revercomb","Reyburn","Reynolds",
    "Rhea","Rhoades","Rhodes","Rhyne","Ribicoff","Rice","Rich","Richard","Richards","Richardson",
    "Richberg","Richmond","Ricketts","Rickey","Riddell","Riddle","Ridenour","Ridgely","Ridgeway","Ridgway",
    "Ridington","Riegel","Rigby","Riggs","Riker","Riley","Rinehart","Ring","Ringling","Ripley",
    "Risley","Ritchie","Rittenhouse","Ritter","Rives","Roach","Roan","Robb","Robbins","Roberts",
    "Robertson","Robeson","Robinson","Roby","Roche","Rochford","Rockefeller","Rockwell","Rodgers","Rodman",
    "Rodney","Roebling","Rogers","Roget","Rohde","Rolfe","Rollins","Rolph","Roman","Rood",
    "Rooney","Roosevelt","Root","Roper","Rorer","Rosa","Roscoe","Rose","Rosecrans","Rosen",
    "Rosenberg","Rosenblatt","Rosendahl","Rosenwald","Rosewater","Ross","Rosser","Rossiter","Roth","Rothschild",
    "Rourke","Rouse","Roush","Routt","Rowan","Rowe","Rowell","Rowland","Rowley","Roy",
    "Royall","Royce","Royer","Rubenstein","Rubin","Rucker","Rudd","Rudge","Rue","Ruffin",
    "Ruffner","Ruger","Ruggles","Runkle","Runyon","Rupp","Rush","Rusk","Ruskin","Russ",
    "Russell","Rustin","Rutgers","Ruth","Rutherford","Rutland","Rutledge","Ryan","Ryder","Ryerson",
    "Rynearson","Ryskind","Sabath","Sabin","Sabine","Sackett","Sackler","Sadler","Safford","Sage",
    "Sailer","Saint","Salem","Salisbury","Salmon","Saltonstall","Sample","Sampson","Samuels","Sandburg",
    "Sanders","Sanderson","Sandford","Sands","Sandusky","Sanger","Santayana","Sappington","Sarat","Sargent",
    "Sarnoff","Sartain","Sartwell","Sasser","Satterfield","Saul","Saunders","Savage","Savell","Savidge",
    "Saville","Sawyer","Saxe","Saxton","Say","Saylor","Sayles","Sayre","Scales","Scammell",
    "Scammon","Scanlon","Scarborough","Scarlett","Schaffer","Schall","Scharps","Schell","Schermerhorn","Schick",
    "Schieffelin","Schiff","Schiller","Schley","Schlitz","Schmidt","Schoellkopf","Schoen","Schofield","Scholle",
    "Schroeder","Schurz","Schuyler","Schwab","Schwarz","Schweitzer","Scofield","Scoggins","Scollard","Scott",
    "Scovel","Scoville","Scribner","Scruggs","Scudder","Scully","Seabright","Seabrook","Seager","Seagrave",
    "Searcy","Sears","Seaton","Seaver","Seawell","Sedgwick","Seeger","Seeley","Seely","Segar",
    "Seibels","Seitz","Selden","Self","Sellers","Seltzer","Selznick","Semmes","Semple","Senff",
    "Senior","Sergeant","Sessions","Seton","Severance","Sevier","Seward","Sewell","Sexton","Seymour",
    "Shackelford","Shadwell","Shafer","Shaftesbury","Shakespeare","Shaler","Shanahan","Shank","Shankland","Shannon",
    "Shapleigh","Sharpe","Shattuck","Shaughnessy","Shaw","Shays","Shea","Sheafe","Shearer","Sheean",
    "Sheehan","Sheffield","Sheldon","Shellabarger","Shelton","Shepard","Shepherd","Shepley","Sheppard","Sherburne",
    "Sheridan","Sherill","Sherman","Sherrod","Sherwood","Shields","Shillito","Shipherd","Shipp","Shippen",
    "Shipstead","Shirk","Shirley","Shockley","Shoemaker","Shore","Short","Shorter","Shortridge","Shoup",
    "Shouse","Shriver","Shufeldt","Shuler","Shull","Shultz","Shumway","Shurtleff","Shute","Sias",
    "Sibley","Sickles","Sidell","Sidney","Siegel","Sigsbee","Sikes","Silliman","Silver","Silvester",
    "Simkins","Simmons","Simms","Simon","Simonds","Simons","Simpson","Sims","Sinclair","Singel",
    "Singleton","Sinnott","Sioussat","Sipple","Sirk","Sisson","Sitgreaves","Sitterly","Sizer","Skaggs",
    "Skeel","Skelton","Skiles","Skinner","Skipwith","Slack","Slade","Slater","Slayton","Sleeper",
    "Slemp","Slessinger","Sloan","Sloane","Slocum","Sloss","Slosson","Small","Smalley","Smart",
    "Smead","Smellie","Smiley","Smith","Smithers","Smoot","Smylie","Smyth","Smythe","Snedeker",
    "Snell","Snelling","Snyder","Sobel","Sockman","Sohier","Soldiers","Solem","Solis","Sollers",
    "Solomon","Solon","Soper","Sorenson","Sorley","Sothoron","Sousa","South","Southall","Southard",
    "Souther","Southgate","Southwick","Sowle","Spalding","Spangler","Sparkman","Sparks","Spaulding","Spear",
    "Spearman","Spears","Speed","Speer","Speers","Spellman","Spence","Spencer","Sperry","Spiegel",
    "Spiller","Spingarn","Spofford","Sprague","Spreckels","Springer","Sproul","Spruance","Spurr","Squires",
    "Stackpole","Stacy","Staigg","Stalvey","Stambaugh","Stamp","Stanard","Standish","Stanfield","Stanford",
    "Stangeland","Stanier","Staniford","Stanley","Stans","Stansbury","Stanton","Staples","Stapleton","Starrett",
    "Starr","Stassen","States","Staunton","Stavisky","Stearns","Stebbins","Stedman","Steedman","Steel",
    "Steele","Steelman","Steers","Stein","Steinbeck","Steiner","Stengel","Stephens","Stephenson","Sterling",
    "Stern","Sterne","Stetson","Steuart","Stevens","Stevenson","Steward","Stewart","Stickney","Stigler",
    "Stillman","Stillwell","Stimson","Stine","Stinson","Stirling","Stockard","Stockbridge","Stockton","Stoddard",
    "Stoeffel","Stokes","Stoll","Stone","Stoneman","Storer","Storey","Storm","Storrow","Storrs",
    "Story","Stotesbury","Stoughton","Stout","Stowe","Strachan","Stranahan","Strange","Stratton","Straus",
    "Strauss","Strawbridge","Street","Streeter","Stribling","Strickland","Strong","Strother","Stroud","Strout",
    "Struble","Stryker","Stuart","Stubblefield","Stubbs","Stuck","Sturgis","Sturtevant","Stutz","Suarez",
    "Sugrue","Sullens","Sullivan","Sully","Sulzberger","Summerall","Summerfield","Summers","Sumner","Sumter",
    "Sunderland","Surrey","Sutcliffe","Sutherland","Sutherland","Sutter","Suttner","Sutton","Suydam","Swain",
    "Swan","Swank","Swann","Swanwick","Swanson","Swartwout","Swayne","Swayze","Swearingen","Sweeney",
    "Sweet","Swenson","Sweitzer","Swift","Swinburne","Swindell","Swope","Sykes","Sylvester","Symmes",
    "Symonds","Symons","Tabb","Tabor","Taft","Talcott","Taliaferro","Tallmadge","Talmadge","Talmage",
    "Tams","Taney","Tanzer","Tappan","Tarbell","Tarkington","Tate","Tatum","Taussig","Tawney",
    "Taylor","Tayntor","Teague","Teale","Tebbets","Telfair","Teller","Templeton","Tenney","Tennyson",
    "Terhune","Terrell","Terrill","Terry","Tewksbury","Thacher","Thackara","Thacker","Thacher","Tharp",
    "Thatcher","Thaw","Thayer","Theobald","Theriault","Thibeault","Thirkield","Thomas","Thompson","Thomson",
    "Thorn","Thorndike","Thorne","Thornton","Thorp","Thorpe","Throckmorton","Thurber","Thurman","Thurmond",
    "Thurston","Tibbits","Tibbs","Ticknor","Tiedeman","Tiffany","Tilden","Tileston","Tilghman","Tillinghast",
    "Tillman","Tilney","Tilton","Timmonds","Tinkham","Tinney","Tinsley","Tipper","Tipton","Titus",
    "Tobey","Tobias","Tobin","Todd","Toland","Tolbert","Toler","Tolles","Tomlinson","Tompkins",
    "Tone","Toney","Tonry","Toole","Toombs","Torbert","Torrey","Totten","Tousey","Tower",
    "Towers","Towne","Townsend","Tracy","Treadwell","Treat","Trego","Trelease","Tremain","Trenchard",
    "Trent","Trescott","Trevelyan","Tribble","Trimble","Tripp","Trist","Trotter","Troup","Trowbridge",
    "Truax","Trudeau","Truesdale","Truett","Truman","Trumbull","Tryon","Tubman","Tuck","Tucker",
    "Tufts","Tully","Tumulty","Tunney","Turnbull","Turner","Turnure","Turney","Turpin","Tuttle",
    "Tuve","Twain","Twichell","Tyler","Tyndale","Tyner","Tyrrell","Udall","Uhler","Ullman",
    "Ullmann","Underhill","Underwood","Upham","Upson","Upton","U'Ren","Urner","Usher","Utley",
    "Vail","Valentine","Vallandigham","Van Anda","Van Antwerp","Van Arsdale","Van Buren","Van Cleve","Van Cortlandt","Van Dam",
    "Vandenberg","Vanderbilt","Van der Donck","Vandergrift","Vanderlip","Vandeventer","Van Devanter","Van Doren","Van Dusen","Van Dyck",
    "Van Dyke","Vane","Van Hise","Van Horne","Van Loon","Van Ness","Van Pelt","Van Rensselaer","Van Sant","Van Sickle",
    "Van Sweringen","Van Vechten","Van Wagenen","Van Zandt","Vardaman","Vare","Varian","Varnum","Vassar","Vaughan",
    "Vaughn","Veiller","Venable","Vera","Verity","VerMeer","Vernon","Ver Planck","Vesey","Vest",
    "Vidal","Viele","Villard","Vinal","Vincent","Vine","Vines","Vinson","Vinton","Voegeli",
    "Vogel","Voight","Volk","Vollmer","Vonnegut","Voorhees","Vorse","Vosper","Vrooman","Wachter",
    "Waddell","Waddingham","Wade","Wadleigh","Wadsworth","Wagner","Wagstaff","Wait","Waite","Wakefield",
    "Walcott","Walden","Waldman","Waldo","Waldorf","Walford","Walker","Wall","Wallace","Wallenstein",
    "Waller","Wallis","Walpole","Walsh","Walter","Walters","Walton","Wander","Wangenheim","Wannamaker",
    "Warburg","Ward","Warden","Wardlaw","Wardwell","Ware","Warfield","Waring","Warner","Warren",
    "Warwick","Washburn","Washington","Waterbury","Waterhouse","Waterman","Waters","Watkins","Watkinson","Watrous",
    "Watson","Watterson","Wattles","Watts","Waugh","Waxman","Wayland","Waymack","Wayne","Wead",
    "Weatherby","Weathers","Weaver","Webb","Webber","Weber","Webster","Weed","Weeks","Wehle",
    "Weicker","Weidman","Weil","Weir","Weis","Weise","Weiss","Welch","Weld","Welker",
    "Welles","Wellman","Wells","Welsh","Wendell","Wentworth","Werner","Wertenbaker","Wescott","Wesley",
    "West","Westcott","Western","Westinghouse","Westlake","Weston","Wetherby","Wetmore","Weyerhaeuser","Whalen",
    "Whaley","Wharton","Wheatland","Wheaton","Wheeler","Wheelock","Wheelwright","Whelan","Whipple","Whitaker",
    "Whitcomb","White","Whitehead","Whitehill","Whitehouse","Whitelaw","Whiteman","Whitfield","Whiting","Whitlock",
    "Whitman","Whitmarsh","Whitney","Whitson","Whittaker","Whittemore","Whitten","Whittier","Whittington","Wickersham",
    "Wickes","Widener","Widnall","Wiegand","Wieland","Wigfall","Wiggin","Wigglesworth","Wight","Wilber",
    "Wilbourn","Wilbur","Wilcox","Wild","Wilder","Wiley","Wilhelm","Wilkes","Wilkie","Wilkins",
    "Wilkinson","Willard","Willcox","Willett","Willey","Williams","Williamson","Williford","Willing","Willis",
    "Williston","Wills","Willkie","Willys","Wilmer","Wilson","Wiltse","Wiman","Wimberly","Wimsatt",
    "Winant","Winchell","Winchester","Windom","Windsor","Wines","Winfrey","Wing","Winslow","Winston",
    "Winter","Winters","Winthrop","Winton","Wirt","Wirth","Wisdom","Wise","Wisner","Wister",
    "Withers","Witherspoon","Withrow","Witmer","Wolcott","Wold","Wolf","Wolfe","Wolff","Wolfgang",
    "Wolford","Woll","Wolman","Wolsey","Wood","Woodbury","Woodford","Woodhull","Woodin","Woodruff",
    "Woods","Woodson","Woodward","Woodworth","Woolsey","Woolworth","Wooten","Worcester","Worden","Work",
    "Workman","Worrell","Worthington","Wren","Wright","Wriston","Wroth","Wu","Wurts","Wyant",
    "Wyatt","Wycherley","Wyer","Wyeth","Wylie","Wyllis","Wyman","Wymore","Wyndham","Wynn",
    "Wynne","Wyse","Yancey","Yandell","Yarborough","Yates","Yeager","Yeaman","Yeardley","Yeates",
    "Yell","Yerkes","Yoakum","Yoder","York","Yorke","Yost","Young","Younger","Younglove",
    "Youngs","Yule","Zabriskie","Zacher","Zane","Zehnder","Zeidler","Zeller","Zenger","Ziegler",
    "Zimmerman","Zink","Zinsser","Zobel","Zorach","Zuckerman","Zug","Zuppke","Zwick"
];

const SKILL_NAME_PREFIXES: &[&str] = &[
    "Blade","Fire","Frost","Thunder","Shadow","Light","Wind","Earth","Blood","Iron",
    "Holy","Dark","Rapid","Piercing","Savage","Burning","Freezing","Cleansing","Healing","Shielding",
    "Swift","Mighty","Venomous","Vengeful","Guardian","Ancient","Arcane","Chaos","Celestial","Demonic",
    "Solar","Lunar","Stellar","Phantom","Crystal","Storm","Ember","Frostbite","Soul","Spirit",
    "Dragon","Phoenix","Titan","Serpent","Wolf","Eagle","Bear","Lion","Raven","Falcon",
    "Iron","Steel","Silver","Golden","Diamond","Obsidian","Emerald","Ruby","Sapphire","Jade",
    "Crimson","Azure","Emerald","Violet","Amber","Ivory","Ebony","Scarlet","Indigo","Bronze",
    "Raging","Calm","Fierce","Gentle","Deadly","Stunning","Blinding","Crushing","Slicing","Piercing"
];

const SKILL_ROOTS: &[&str] = &[
    "Strike","Slash","Blast","Wave","Burst","Beam","Storm","Bolt","Strike","Thrust",
    "Cut","Blow","Shot","Volley","Barrage","Flurry","Salvo","Storm","Wave","Surge",
    "Roar","Shout","Cry","Call","Song","Chant","Whisper","Roar","Howl","Growl",
    "Ward","Shield","Barrier","Aegis","Wall","Guard","Protection","Shell","Veil","Cloak",
    "Touch","Heal","Mend","Restore","Revive","Renew","Purify","Cleanse","Bless","Grace",
    "Dash","Charge","Leap","Rush","Sprint","Swift","Flash","Blink","Step","Surge",
    "Mark","Brand","Curse","Hex","Bane","Poison","Plague","Toxin","Venom","Corruption",
    "Dance","Form","Stance","Posture","Flow","Weave","Thread","Knot","Spiral","Circle",
    "Rage","Fury","Wrath","Frenzy","Madness","Berserk","Uprising","Rebellion","Onslaught","Assault",
    "Wisdom","Intellect","Mind","Thought","Focus","Clarity","Insight","Vision","Dream","Memory",
    "Steal","Pillage","Rip","Tear","Rend","Cleave","Sever","Slice","Gash","Lacerate",
    "Echo","Reverb","Resonance","Vibration","Pulse","Throb","Beat","Rhythm","Cadence","Tempo"
];

const SKILL_SUFFIXES: &[&str] = &[
    "of Dawn","of Dusk","of Night","of Day","of Winter","of Summer","of Spring","of Autumn",
    "of Fire","of Ice","of Lightning","of Wind","of Earth","of Water","of Light","of Shadow",
    "of the Bear","of the Wolf","of the Eagle","of the Dragon","of the Phoenix","of the Serpent",
    "of Valor","of Courage","of Hope","of Despair","of Wrath","of Mercy","of Wisdom","of Fury",
    "of Kings","of Lords","of Heroes","of Legends","of the Ancients","of Ages Past","of Eternity",
    "of Battle","of War","of Victory","of Defeat","of Conquest","of Triumph","of Glory",
    "Strike","Blade","Fury","Wrath","Song","Roar","Dance","Grace","Pride","Doom"
];

fn gen_skill_name(seed_val: u32, rng: &mut crate::rng::SeededRng) -> String {
    let prefix = SKILL_NAME_PREFIXES[(seed_val as usize) % SKILL_NAME_PREFIXES.len()];
    let root_idx = rng.gen_range(0, SKILL_ROOTS.len() as i32 - 1) as usize;
    let root = SKILL_ROOTS[root_idx];
    if rng.gen_bool(0.3) {
        let suffix_idx = rng.gen_range(0, SKILL_SUFFIXES.len() as i32 - 1) as usize;
        format!("{} {} {}", prefix, root, SKILL_SUFFIXES[suffix_idx])
    } else {
        format!("{} {}", prefix, root)
    }
}

fn gen_character(id: u32) -> CharacterTemplate {
    let mut rng = crate::rng::SeededRng::new(id as u64 * 7919 + 104729);

    let first = FIRST_NAMES[(id as usize) % FIRST_NAMES.len()];
    let last = LAST_NAMES[((id as usize) * 31 + 17) % LAST_NAMES.len()];
    let name = format!("{} {}", first, last);

    let unit_type = match rng.gen_range(0, 3) {
        0 => UnitType::Infantry,
        1 => UnitType::Cavalry,
        2 if rng.gen_bool(0.3) => UnitType::Flying,
        2 => UnitType::Swimming,
        _ => UnitType::Infantry,
    };

    let move_range = match unit_type {
        UnitType::Infantry => 3,
        UnitType::Cavalry => 5,
        UnitType::Flying => 4,
        UnitType::Swimming => 4,
    };

    let preferred_position = match rng.gen_range(0, 2) {
        0 => Position::Frontline,
        1 => Position::Midline,
        _ => Position::Backline,
    };

    let normal_name = format!("{} Strike", first);
    let normal_target = match rng.gen_range(0, 4) {
        0 => SkillTarget::EnemyFront,
        1 => SkillTarget::EnemyLowestHp,
        2 => SkillTarget::EnemyHighestAtk,
        _ => SkillTarget::EnemyAll,
    };
    let normal_multi = 0.8 + rng.gen_percent() * 0.4;
    let normal_attack = Skill::normal_attack(&normal_name, normal_multi, normal_target);

    let move_skill = Skill::move_skill(
        match unit_type {
            UnitType::Infantry => "Infantry Move",
            UnitType::Cavalry => "Cavalry Move",
            UnitType::Flying => "Flying Move",
            UnitType::Swimming => "Swim Move",
        },
        move_range,
    );

    // SP skill (most characters have one)
    let sp_skill = if rng.gen_bool(0.85) {
        let sk_name = gen_skill_name(id, &mut rng);
        let effs = match rng.gen_range(0, 6) {
            0 => vec![SkillEffect::Damage { multiplier: 1.3 + rng.gen_percent() * 0.5, target: SkillTarget::EnemyFront }],
            1 => vec![SkillEffect::Damage { multiplier: 0.8, target: SkillTarget::EnemyAll },
                       SkillEffect::DebuffDef { percent: 0.15, duration: 2, target: SkillTarget::EnemyAll }],
            2 => vec![SkillEffect::Heal { multiplier: 0.5 + rng.gen_percent() * 0.5, target: SkillTarget::AllyLowestHp }],
            3 => vec![SkillEffect::BuffAtk { percent: 0.20, duration: 2, target: SkillTarget::AllAllies }],
            4 => vec![SkillEffect::Shield { amount: 30 + rng.gen_range(0, 40), target: SkillTarget::Self_ }],
            _ => vec![SkillEffect::Damage { multiplier: 1.5, target: SkillTarget::EnemyHighestAtk },
                       SkillEffect::EnergyGain { amount: 20, target: SkillTarget::Self_ }],
        };
        Some(Skill::sp_skill(&sk_name, &format!("{} skill point ability", sk_name), effs))
    } else {
        None
    };

    // CD skill
    let cd_skill = if rng.gen_bool(0.70) {
        let sk_name = gen_skill_name(id.wrapping_add(100), &mut rng);
        let cd = rng.gen_range(2, 5);
        let effs = match rng.gen_range(0, 5) {
            0 => vec![SkillEffect::Damage { multiplier: 1.8 + rng.gen_percent() * 0.8, target: SkillTarget::EnemyAll }],
            1 => vec![SkillEffect::BuffDef { percent: 0.30, duration: 3, target: SkillTarget::AllAllies }],
            2 => vec![SkillEffect::Heal { multiplier: 0.8, target: SkillTarget::AllyAll }],
            3 => vec![SkillEffect::Stun { duration: 1, target: SkillTarget::EnemyFront }],
            _ => vec![SkillEffect::AoEDamage { multiplier: 1.2 + rng.gen_percent() * 0.6 }],
        };
        Some(Skill::cd_skill(&sk_name, &format!("{} cooldown ability ({} CD)", sk_name, cd), cd, effs))
    } else {
        None
    };

    // Energy skill
    let energy_skill = if rng.gen_bool(0.60) {
        let sk_name = gen_skill_name(id.wrapping_add(200), &mut rng);
        let effs = match rng.gen_range(0, 5) {
            0 => vec![SkillEffect::Damage { multiplier: 2.5 + rng.gen_percent(), target: SkillTarget::EnemyAll }],
            1 => vec![SkillEffect::Heal { multiplier: 1.5, target: SkillTarget::AllyAll }],
            2 => vec![SkillEffect::BuffSpd { percent: 0.50, duration: 3, target: SkillTarget::AllAllies }],
            3 => vec![SkillEffect::Revive { hp_percent: 0.5, target: SkillTarget::AllyFront }],
            _ => vec![SkillEffect::Cleanse { target: SkillTarget::AllAllies },
                       SkillEffect::Heal { multiplier: 0.5, target: SkillTarget::AllyAll }],
        };
        Some(Skill::energy_skill(&sk_name, &format!("{} energy ability", sk_name), effs))
    } else {
        None
    };

    // Ultimate
    let ultimate = if rng.gen_bool(0.50) {
        let sk_name = gen_skill_name(id.wrapping_add(300), &mut rng);
        let effs = match rng.gen_range(0, 4) {
            0 => vec![SkillEffect::Damage { multiplier: 4.0, target: SkillTarget::EnemyAll }],
            1 => vec![SkillEffect::Heal { multiplier: 2.0, target: SkillTarget::AllyAll },
                       SkillEffect::BuffAtk { percent: 0.50, duration: 3, target: SkillTarget::AllAllies }],
            2 => vec![SkillEffect::Damage { multiplier: 5.0, target: SkillTarget::EnemyHighestAtk }],
            _ => vec![SkillEffect::Revive { hp_percent: 1.0, target: SkillTarget::AllyAll },
                       SkillEffect::Heal { multiplier: 1.0, target: SkillTarget::AllyAll }],
        };
        Some(Skill::ultimate(&sk_name, &format!("{} ultimate (once per battle)", sk_name), effs))
    } else {
        None
    };

    CharacterTemplate {
        id,
        name,
        unit_type,
        base_atk: 100,
        base_hp: 100,
        base_spd: 100,
        preferred_position,
        normal_attack,
        move_skill,
        sp_skill,
        cd_skill,
        energy_skill,
        ultimate,
    }
}

/// Get all 1000 character templates (generated procedurally on first call).
pub fn all_templates() -> Vec<CharacterTemplate> {
    (0..1000).map(gen_character).collect()
}

/// Find a template by name.
pub fn find_template(name: &str) -> Option<CharacterTemplate> {
    all_templates().into_iter().find(|t| t.name.eq_ignore_ascii_case(name))
}

/// Get a template by index (0..1000).
pub fn template_by_id(id: u32) -> CharacterTemplate {
    gen_character(id)
}
