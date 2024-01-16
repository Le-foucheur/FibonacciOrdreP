#let question = counter("questions")
#set heading(
  numbering: (..numbers) => {
    let n = numbers.pos().len();
    if n == 1 {numbering("1.", numbers.pos().at(0)) } 
    else if n == 2 { [Partie ]; numbering("I", numbers.pos().at(1)) ;"." } else if n == 3 { [N=°];question.step(); question.display();"." } 
    else if n == 4 { numbering("a.", numbers.pos().at(3) + 1) } 
    else if n == 5 { numbering("i.", numbers.pos().at(4)) }
  },
)
#show heading : it => {
  if it.body == [] or it.level >= 3 [#parbreak() #linebreak() #counter(heading).display() #it.body] else [#parbreak() #linebreak() #counter(heading).display() #it.body]
}

#align(center, text(20pt)[TITRE])

#align(center,text[= Introduction])

La suite de Fibonacci a tout d'abord été étudiée en Inde via un problème de combinatoire dans des sortes de poèmes au V#super("e") siècle avant J.-C. par Pingala @Pingala. Puis, elle a été étudiée en Italie par le célèbre Léonard de Pise, plus connu sous le nom de Fibonacci, dans un problème sur la taille d'une population de lapins apparu dans son ouvrage #text(style: "italic")[Liber abbaci] @Liber en 1202.\
Cette suite auras toujours créé un engouement, et donc énormément de généralisation ont été créé comme les suites de Lucas@Lucas.\
Mais parmis toutes ces généralisations beaucoup sont laissé de coté, et nous allons nous intéréser à l'une de celle-ci.

#bibliography("Bibli.yml", style: "biomed-central")