open Oplot.Plt

let n = 100;;
let p = 1.;;

let rec facto n = 
  let exception N_negatif in
  match n with
  | 0 -> 1
  | _ when n > 0 -> n * facto (n-1)
  | _ -> raise N_negatif
;;

let h k x =
  let rec aux k x =
    match k with
    | 0 -> 1.
    | _ -> (x -. (float_of_int (k-1)) ) *. (aux  (k-1) x)
  in
  1. /. (float_of_int (facto k)) *. (aux k x) 

;;

(*let pash = Hashtbl.create n ;;*)

let rec poly p k x = 
  let exception Erreur_de_reccurtion_f in
  let rec aux i acc = 
    if i >= 0 then
      aux (i-1) (acc +. ( poly p i ( float_of_int(i+1) *. p ) ) *. h (k - 1 - i) ( x -. (float_of_int k) *. p))
    else
      0.
  in 
  (h k ( x -. (float_of_int k) *. p)) +. (aux (k-1) 0.)

let rec f p x =  
  poly p (floor (x /. p) |> int_of_float) x
;;


(*for i = 0 to 20 do
  float_of_int i |>  f 1. |> Printf.printf "%f\n"
done;;*)

let a = axis 0. 0.;;
let p = plot (f 1.) (0.) 5.;;

display [ Color red; p; Color black; a ];;