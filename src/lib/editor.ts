


// inline editor for panel
export const apiKey = "on3m91jkkv1mbc3fe8ez60gsyv4yt9yv02cx3hmzixd44cwu";
export const text_patterns = [
    { start: '#', format: 'h1' },
    { start: '##', format: 'h2' },
    { start: '###', format: 'h3' },
    { start: '####', format: 'h4' },
    { start: '#####', format: 'h5' },
    { start: '######', format: 'h6' },
    { start: '* ', cmd: 'InsertUnorderedList' },
    { start: '- ', cmd: 'InsertUnorderedList' },
    // { start: ' $$', end: '$$ ', cmd: 'math' },
    { start: '*', end: '*', format: 'italic' },
    { start: '**', end: '**', format: 'bold' },
    { start: '`', end: '`', format: 'code' },
    { start: '1. ', cmd: 'InsertOrderedList', value: { 'list-style-type': 'decimal' } },
    { start: '1) ', cmd: 'InsertOrderedList', value: { 'list-style-type': 'decimal' } },
    { start: 'a. ', cmd: 'InsertOrderedList', value: { 'list-style-type': 'lower-alpha' } },
    { start: 'a) ', cmd: 'InsertOrderedList', value: { 'list-style-type': 'lower-alpha' } },
    { start: 'i. ', cmd: 'InsertOrderedList', value: { 'list-style-type': 'lower-roman' } },
    { start: 'i) ', cmd: 'InsertOrderedList', value: { 'list-style-type': 'lower-roman' } },
    { start: '---', replacement: '<hr/>' },
    { start: '--', replacement: '—' },
    { start: ' >>', replacement: ' →'},
    { start: '==>', replacement: '⇒'}
    // { start: ' ', replacement: ' ↓<br>'}
    // { start: '`', replacement: '<hr/>' }
]

export const preprocess = (editor: any, args: any) => {
        
        args.content = args.content.replaceAll(" &gt;&gt;", ' →');
        args.content = args.content.replaceAll(">* ", '>• ');
        args.content = args.content.replaceAll("==>", '⇒');
        if (args.content[0] == "*")
            args.content = args.content.replace("*", '•');
        args.content = args.content.replaceAll(" -- ", " — ")

        // need to put whitespaces with tabs (try this in paste_postprocess)
        // args.content = args.content.replaceAll("</span>•", ' </span><span style="margin-left: 40px;"></span>•');
        // args.content = args.content.replaceAll("</span>&bull;", `</span><div style="margin-left: 40px;"></div>&bull;`)
        


        // apply code ( punctuation: [\s|;|,|.|?|<] )
        // style="background-color: #FAEBDD;"
        let results = args.content.match(/\s`\S(.*?)`/g);
        if (results)
            for (const result of results) {
                const codestr = ' <code>' + result.substring(2, result.length-1) + "</code>";
                args.content = args.content.replaceAll(result, codestr);
            }

        // bold
        results = args.content.match(/\s\*\*\S(.*?)\*\*/g);
        if (results)
            for (const result of results) {
                const codestr = ' <b>' + result.substring(3, result.length-2) + "</b>";
                args.content = args.content.replaceAll(result, codestr);
            }
        // italic
        results = args.content.match(/\s\*\S(.*?)\*/g);
        if (results)
            for (const result of results) {
                const codestr = ' <i>' + result.substring(2, result.length-1) + "</i>";
                args.content = args.content.replaceAll(result, codestr);
            }

        // match strings inside of double $ as math, and render as katex
        // let results = args.content.match(/\$\$(.*?)\$\$/g);
        // if (results != null)
        // 	for (let result of results) {
        // 		const mathstr = math(result.substring(2, result.length-2));
        // 		args.content = args.content + mathstr;
        // 	}

    }

// export default { preprocess as "preprocess", text_patterns as "text_patterns", apiKey };