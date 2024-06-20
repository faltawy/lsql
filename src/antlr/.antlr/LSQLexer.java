// Generated from e://side_projects//lsql//src//antlr//LSQLexer.g4 by ANTLR 4.13.1
import org.antlr.v4.runtime.Lexer;
import org.antlr.v4.runtime.CharStream;
import org.antlr.v4.runtime.Token;
import org.antlr.v4.runtime.TokenStream;
import org.antlr.v4.runtime.*;
import org.antlr.v4.runtime.atn.*;
import org.antlr.v4.runtime.dfa.DFA;
import org.antlr.v4.runtime.misc.*;

@SuppressWarnings({"all", "warnings", "unchecked", "unused", "cast", "CheckReturnValue", "this-escape"})
public class LSQLexer extends Lexer {
	static { RuntimeMetaData.checkVersion("4.13.1", RuntimeMetaData.VERSION); }

	protected static final DFA[] _decisionToDFA;
	protected static final PredictionContextCache _sharedContextCache =
		new PredictionContextCache();
	public static final int
		SELECT=1, FROM=2, WHERE=3, CREATE=4, FILE=5, CHANGE_DIR=6, EQ=7, LT=8, 
		GT=9, LPAREN=10, RPAREN=11, COMMA=12, SEMI=13, DOT=14, DOTDOT=15, ID=16, 
		NUM=17, WS=18, STRING=19;
	public static String[] channelNames = {
		"DEFAULT_TOKEN_CHANNEL", "HIDDEN"
	};

	public static String[] modeNames = {
		"DEFAULT_MODE"
	};

	private static String[] makeRuleNames() {
		return new String[] {
			"SELECT", "FROM", "WHERE", "CREATE", "FILE", "CHANGE_DIR", "EQ", "LT", 
			"GT", "LPAREN", "RPAREN", "COMMA", "SEMI", "DOT", "DOTDOT", "ID", "NUM", 
			"WS", "STRING"
		};
	}
	public static final String[] ruleNames = makeRuleNames();

	private static String[] makeLiteralNames() {
		return new String[] {
			null, "'SELECT'", "'FROM'", "'WHERE'", "'CREATE'", "'FILE'", "'CD'", 
			"'='", "'<'", "'>'", "'('", "')'", "','", "';'", "'.'", "'..'"
		};
	}
	private static final String[] _LITERAL_NAMES = makeLiteralNames();
	private static String[] makeSymbolicNames() {
		return new String[] {
			null, "SELECT", "FROM", "WHERE", "CREATE", "FILE", "CHANGE_DIR", "EQ", 
			"LT", "GT", "LPAREN", "RPAREN", "COMMA", "SEMI", "DOT", "DOTDOT", "ID", 
			"NUM", "WS", "STRING"
		};
	}
	private static final String[] _SYMBOLIC_NAMES = makeSymbolicNames();
	public static final Vocabulary VOCABULARY = new VocabularyImpl(_LITERAL_NAMES, _SYMBOLIC_NAMES);

	/**
	 * @deprecated Use {@link #VOCABULARY} instead.
	 */
	@Deprecated
	public static final String[] tokenNames;
	static {
		tokenNames = new String[_SYMBOLIC_NAMES.length];
		for (int i = 0; i < tokenNames.length; i++) {
			tokenNames[i] = VOCABULARY.getLiteralName(i);
			if (tokenNames[i] == null) {
				tokenNames[i] = VOCABULARY.getSymbolicName(i);
			}

			if (tokenNames[i] == null) {
				tokenNames[i] = "<INVALID>";
			}
		}
	}

	@Override
	@Deprecated
	public String[] getTokenNames() {
		return tokenNames;
	}

	@Override

	public Vocabulary getVocabulary() {
		return VOCABULARY;
	}


	public LSQLexer(CharStream input) {
		super(input);
		_interp = new LexerATNSimulator(this,_ATN,_decisionToDFA,_sharedContextCache);
	}

	@Override
	public String getGrammarFileName() { return "LSQLexer.g4"; }

	@Override
	public String[] getRuleNames() { return ruleNames; }

	@Override
	public String getSerializedATN() { return _serializedATN; }

	@Override
	public String[] getChannelNames() { return channelNames; }

	@Override
	public String[] getModeNames() { return modeNames; }

	@Override
	public ATN getATN() { return _ATN; }

	public static final String _serializedATN =
		"\u0004\u0000\u0013w\u0006\uffff\uffff\u0002\u0000\u0007\u0000\u0002\u0001"+
		"\u0007\u0001\u0002\u0002\u0007\u0002\u0002\u0003\u0007\u0003\u0002\u0004"+
		"\u0007\u0004\u0002\u0005\u0007\u0005\u0002\u0006\u0007\u0006\u0002\u0007"+
		"\u0007\u0007\u0002\b\u0007\b\u0002\t\u0007\t\u0002\n\u0007\n\u0002\u000b"+
		"\u0007\u000b\u0002\f\u0007\f\u0002\r\u0007\r\u0002\u000e\u0007\u000e\u0002"+
		"\u000f\u0007\u000f\u0002\u0010\u0007\u0010\u0002\u0011\u0007\u0011\u0002"+
		"\u0012\u0007\u0012\u0001\u0000\u0001\u0000\u0001\u0000\u0001\u0000\u0001"+
		"\u0000\u0001\u0000\u0001\u0000\u0001\u0001\u0001\u0001\u0001\u0001\u0001"+
		"\u0001\u0001\u0001\u0001\u0002\u0001\u0002\u0001\u0002\u0001\u0002\u0001"+
		"\u0002\u0001\u0002\u0001\u0003\u0001\u0003\u0001\u0003\u0001\u0003\u0001"+
		"\u0003\u0001\u0003\u0001\u0003\u0001\u0004\u0001\u0004\u0001\u0004\u0001"+
		"\u0004\u0001\u0004\u0001\u0005\u0001\u0005\u0001\u0005\u0001\u0006\u0001"+
		"\u0006\u0001\u0007\u0001\u0007\u0001\b\u0001\b\u0001\t\u0001\t\u0001\n"+
		"\u0001\n\u0001\u000b\u0001\u000b\u0001\f\u0001\f\u0001\r\u0001\r\u0001"+
		"\u000e\u0001\u000e\u0001\u000e\u0001\u000f\u0001\u000f\u0005\u000f^\b"+
		"\u000f\n\u000f\f\u000fa\t\u000f\u0001\u0010\u0004\u0010d\b\u0010\u000b"+
		"\u0010\f\u0010e\u0001\u0011\u0004\u0011i\b\u0011\u000b\u0011\f\u0011j"+
		"\u0001\u0011\u0001\u0011\u0001\u0012\u0001\u0012\u0005\u0012q\b\u0012"+
		"\n\u0012\f\u0012t\t\u0012\u0001\u0012\u0001\u0012\u0001r\u0000\u0013\u0001"+
		"\u0001\u0003\u0002\u0005\u0003\u0007\u0004\t\u0005\u000b\u0006\r\u0007"+
		"\u000f\b\u0011\t\u0013\n\u0015\u000b\u0017\f\u0019\r\u001b\u000e\u001d"+
		"\u000f\u001f\u0010!\u0011#\u0012%\u0013\u0001\u0000\u0004\u0003\u0000"+
		"AZ__az\u0004\u000009AZ__az\u0001\u000009\u0003\u0000\t\n\r\r  z\u0000"+
		"\u0001\u0001\u0000\u0000\u0000\u0000\u0003\u0001\u0000\u0000\u0000\u0000"+
		"\u0005\u0001\u0000\u0000\u0000\u0000\u0007\u0001\u0000\u0000\u0000\u0000"+
		"\t\u0001\u0000\u0000\u0000\u0000\u000b\u0001\u0000\u0000\u0000\u0000\r"+
		"\u0001\u0000\u0000\u0000\u0000\u000f\u0001\u0000\u0000\u0000\u0000\u0011"+
		"\u0001\u0000\u0000\u0000\u0000\u0013\u0001\u0000\u0000\u0000\u0000\u0015"+
		"\u0001\u0000\u0000\u0000\u0000\u0017\u0001\u0000\u0000\u0000\u0000\u0019"+
		"\u0001\u0000\u0000\u0000\u0000\u001b\u0001\u0000\u0000\u0000\u0000\u001d"+
		"\u0001\u0000\u0000\u0000\u0000\u001f\u0001\u0000\u0000\u0000\u0000!\u0001"+
		"\u0000\u0000\u0000\u0000#\u0001\u0000\u0000\u0000\u0000%\u0001\u0000\u0000"+
		"\u0000\u0001\'\u0001\u0000\u0000\u0000\u0003.\u0001\u0000\u0000\u0000"+
		"\u00053\u0001\u0000\u0000\u0000\u00079\u0001\u0000\u0000\u0000\t@\u0001"+
		"\u0000\u0000\u0000\u000bE\u0001\u0000\u0000\u0000\rH\u0001\u0000\u0000"+
		"\u0000\u000fJ\u0001\u0000\u0000\u0000\u0011L\u0001\u0000\u0000\u0000\u0013"+
		"N\u0001\u0000\u0000\u0000\u0015P\u0001\u0000\u0000\u0000\u0017R\u0001"+
		"\u0000\u0000\u0000\u0019T\u0001\u0000\u0000\u0000\u001bV\u0001\u0000\u0000"+
		"\u0000\u001dX\u0001\u0000\u0000\u0000\u001f[\u0001\u0000\u0000\u0000!"+
		"c\u0001\u0000\u0000\u0000#h\u0001\u0000\u0000\u0000%n\u0001\u0000\u0000"+
		"\u0000\'(\u0005S\u0000\u0000()\u0005E\u0000\u0000)*\u0005L\u0000\u0000"+
		"*+\u0005E\u0000\u0000+,\u0005C\u0000\u0000,-\u0005T\u0000\u0000-\u0002"+
		"\u0001\u0000\u0000\u0000./\u0005F\u0000\u0000/0\u0005R\u0000\u000001\u0005"+
		"O\u0000\u000012\u0005M\u0000\u00002\u0004\u0001\u0000\u0000\u000034\u0005"+
		"W\u0000\u000045\u0005H\u0000\u000056\u0005E\u0000\u000067\u0005R\u0000"+
		"\u000078\u0005E\u0000\u00008\u0006\u0001\u0000\u0000\u00009:\u0005C\u0000"+
		"\u0000:;\u0005R\u0000\u0000;<\u0005E\u0000\u0000<=\u0005A\u0000\u0000"+
		"=>\u0005T\u0000\u0000>?\u0005E\u0000\u0000?\b\u0001\u0000\u0000\u0000"+
		"@A\u0005F\u0000\u0000AB\u0005I\u0000\u0000BC\u0005L\u0000\u0000CD\u0005"+
		"E\u0000\u0000D\n\u0001\u0000\u0000\u0000EF\u0005C\u0000\u0000FG\u0005"+
		"D\u0000\u0000G\f\u0001\u0000\u0000\u0000HI\u0005=\u0000\u0000I\u000e\u0001"+
		"\u0000\u0000\u0000JK\u0005<\u0000\u0000K\u0010\u0001\u0000\u0000\u0000"+
		"LM\u0005>\u0000\u0000M\u0012\u0001\u0000\u0000\u0000NO\u0005(\u0000\u0000"+
		"O\u0014\u0001\u0000\u0000\u0000PQ\u0005)\u0000\u0000Q\u0016\u0001\u0000"+
		"\u0000\u0000RS\u0005,\u0000\u0000S\u0018\u0001\u0000\u0000\u0000TU\u0005"+
		";\u0000\u0000U\u001a\u0001\u0000\u0000\u0000VW\u0005.\u0000\u0000W\u001c"+
		"\u0001\u0000\u0000\u0000XY\u0005.\u0000\u0000YZ\u0005.\u0000\u0000Z\u001e"+
		"\u0001\u0000\u0000\u0000[_\u0007\u0000\u0000\u0000\\^\u0007\u0001\u0000"+
		"\u0000]\\\u0001\u0000\u0000\u0000^a\u0001\u0000\u0000\u0000_]\u0001\u0000"+
		"\u0000\u0000_`\u0001\u0000\u0000\u0000` \u0001\u0000\u0000\u0000a_\u0001"+
		"\u0000\u0000\u0000bd\u0007\u0002\u0000\u0000cb\u0001\u0000\u0000\u0000"+
		"de\u0001\u0000\u0000\u0000ec\u0001\u0000\u0000\u0000ef\u0001\u0000\u0000"+
		"\u0000f\"\u0001\u0000\u0000\u0000gi\u0007\u0003\u0000\u0000hg\u0001\u0000"+
		"\u0000\u0000ij\u0001\u0000\u0000\u0000jh\u0001\u0000\u0000\u0000jk\u0001"+
		"\u0000\u0000\u0000kl\u0001\u0000\u0000\u0000lm\u0006\u0011\u0000\u0000"+
		"m$\u0001\u0000\u0000\u0000nr\u0005\'\u0000\u0000oq\t\u0000\u0000\u0000"+
		"po\u0001\u0000\u0000\u0000qt\u0001\u0000\u0000\u0000rs\u0001\u0000\u0000"+
		"\u0000rp\u0001\u0000\u0000\u0000su\u0001\u0000\u0000\u0000tr\u0001\u0000"+
		"\u0000\u0000uv\u0005\'\u0000\u0000v&\u0001\u0000\u0000\u0000\u0005\u0000"+
		"_ejr\u0001\u0006\u0000\u0000";
	public static final ATN _ATN =
		new ATNDeserializer().deserialize(_serializedATN.toCharArray());
	static {
		_decisionToDFA = new DFA[_ATN.getNumberOfDecisions()];
		for (int i = 0; i < _ATN.getNumberOfDecisions(); i++) {
			_decisionToDFA[i] = new DFA(_ATN.getDecisionState(i), i);
		}
	}
}