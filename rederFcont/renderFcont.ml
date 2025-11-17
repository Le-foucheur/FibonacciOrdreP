open Oplot.Plt

let n = 30.;;
let p = 20.;;

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
let polyash = int_of_float n |> Hashtbl.create ;;
let hash = int_of_float n |> Hashtbl.create

let rec poly k p x = 
  (* Marche pas, pourquoi ? :/
  let rec aux i acc = 
    if i >= 0 then
      aux (i-1) (acc +. ( poly p i ( float_of_int(i+1) *. p ) ) *. h (k - 1 - i) ( x -. (float_of_int k) *. p))
    else
      0.
  in 
  (h k ( x -. (float_of_int k) *. p)) +. (aux (k-1) 0.)
  *)
  let res = ref 0. in 
  for i = 0 to k-1 do
    if Hashtbl.mem polyash i then
      let tmp = Hashtbl.find polyash i in
      res := !res +. ( tmp p ( float_of_int(i+1) *. p ) ) *. h (k - 1 - i) ( x -. (float_of_int k) *. p)
    else
      let tmp = poly i in
      Hashtbl.add polyash i tmp;
      res := !res +. ( tmp p ( float_of_int(i+1) *. p ) ) *. h (k - 1 - i) ( x -. (float_of_int k) *. p)
  done;
  (h k ( x -. (float_of_int k) *. p)) +. !res
;;

let rec f p x =  
  poly (floor (x /. p) |> int_of_float) p x
;;

let rap n x = (f x (n +. 1.)) /. (f x n)
;;


(*for i = 0 to 20 do
  float_of_int i |>  f 1. |> Printf.printf "i: %d  --  %f\n" i
done;;*)

let a = axis 1. 1.5;;
let p = adapt_plot (rap n) ~step:0.1 (1.) p;;

display [ Color red; p; Color black; a ];;