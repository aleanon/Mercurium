#![cfg(test)]

use bip39::Mnemonic;
use types::crypto::{EncryptedMnemonic, Password};


use crate::credentials::{delete_encrypted_mnemonic, delete_salt, get_db_encryption_salt, get_encrypted_mnemonic, store_db_encryption_salt, store_encrypted_mnemonic};

use super::{delete::tests::*, get_credentials::tests::*, store_credentials::tests::*};

#[test]
fn test_store_get_delete_blob() {
    for i in 0..TEST_TARGETS.len() {
        let target = TEST_TARGETS[i];
        let mut blob = TEST_PASSWORDS[i].as_bytes().to_vec();
        store_blob_test(blob.as_mut_ptr(), blob.len(), target);
        let retrieved_blob = get_blob_test(target);
        assert!(blob == retrieved_blob, "Blob mismatch");
        delete_credentials_test(target)
    }
}

#[test]
fn test_store_get_delete_salt() {
    for i in 0..TEST_PASSWORDS.len() {
        let password = Password::from(TEST_PASSWORDS[i]);
        let (_, salt) = password
            .derive_new_db_encryption_key()
            .expect("Unable to derive key and salt from password");
        store_db_encryption_salt(salt.clone()).expect("Failed to store salt");

        let retrieved_salt = get_db_encryption_salt().expect("Failed when retrieving salt");
        assert!(salt == retrieved_salt, "Salt mismatch");
        delete_salt().expect("Failed to delete salt");
    }
}

#[test]
fn test_store_get_delete_encrypted_mnemonic() {
    for i in 0..TEST_PASSWORDS.len() {
        let password = Password::from(TEST_PASSWORDS[i]);
        let mnemonic = Mnemonic::new(bip39::MnemonicType::Words24, bip39::Language::English);
        let encrypted_mnemonic =
            EncryptedMnemonic::new(&mnemonic, "", &password).expect("Unable to encrypt mnemonic");
        store_encrypted_mnemonic(&encrypted_mnemonic).expect("Failed to store encrypted mnemonic");

        let encrypted_mnemonic =
            get_encrypted_mnemonic().expect("Failed when retrieving encrypted mnemonic");
        let (retrieved_mnemonic, _seed_password) = encrypted_mnemonic
            .decrypt_mnemonic(&password)
            .expect("Failed when decrypting mnemonic");
        assert!(
            mnemonic.phrase() == retrieved_mnemonic.phrase(),
            "Mnemonic mismatch"
        );

        delete_encrypted_mnemonic().expect("Failed to delete encrypted mnemonic")
    }
}

const TEST_TARGETS: [&'static str; 50] = [
    "GouBghTVHe",
    "OVW9EyeAjk",
    "8PTT9wxm5K",
    "9IUMTo9Dxs",
    "BD6RBImkMm",
    "4GGhwyjgMU",
    "hArqBxzh2D",
    "eg47iEME2n",
    "1S9BaaL8AN",
    "1HdmF6582E",
    "jOAjpL27hg",
    "qDlmRMIACt",
    "78fYW9qShq",
    "QGsfVuiNiS",
    "zUnwt0drrd",
    "k16mW00A75",
    "m0UKuyiDdg",
    "SxZVvwuY2V",
    "nBAbkKiXDH",
    "qhdYhMAQmu",
    "QEJ9mDZhsL",
    "p2Mnbhvtn4",
    "iLbMkMv0Va",
    "n07EuwlcVO",
    "6xDMM6g8qZ",
    "szl2TVFLc7",
    "MyNKyeEAVY",
    "NabdlDBrHc",
    "5C8R9Jdz7x",
    "Sat0NoQwBB",
    "GiYzBuEuP9",
    "vkyUXjFQOq",
    "x88Zmai2M5",
    "Im7hJQ25la",
    "NrfoU30RcP",
    "ldyF34vakF",
    "BykR2o554L",
    "oTfvexGYXA",
    "ywdio9WP61",
    "wsOrRADq3v",
    "C0qZIuuh17",
    "73WlQmk1O9",
    "BqLUudhydY",
    "ZHabEW0LgB",
    "yYnxoKENg7",
    "qPADotDrVO",
    "ug7v3R1MVP",
    "Vmc8vT5FIc",
    "fEMYc4rVMg",
    "jRa3PyyXMC",
];

const TEST_PASSWORDS: [&'static str; 50] = [
    "R^+:e@,'8dDjAF]J'-O<]M6X,:joGl)$(nHdL<oEL_cnx6n^-fG",
    "#c&awq@a<(PgV.D{{b?TW9@(-vd\"Nq&3%$_!q",
    "Rsn)<;ez/OFV1\\6P.e[v:2kB1[p)YpQEG837E|_IN:b9PX/yq.HQem8",
    "Pn);oSi\\t4e>_pF_}W4i3nS>t%g}#aK26h7y",
    "fI\"J~NF|8kPticmk-?3-K:Tq'-x/p.v\\P+}<\\t=xQ`WFmb(Zb5P/]",
    "H+:h_ST#[Vx15\"V*_Rq|0]cCX?C1@",
    "&{ZPALe7[{xF$pUZAW~@j!ZpWO3l8aIT%/[KWp]",
    "\"(o=A0WS|}R*.s\\pl#xs@k|c=NA",
    "jTsq=])?nq#'*54%^`oE}`BNqwZ.+*A)nGK1J31",
    "v&=@gKCC[!ixmc!x+liC9x{Y{KE?EOOVU",
    "HGfCv\"uj<9|`+$SyI2x'NqmzBR~dx:fNSam&GmMQjx2t4hhq@=w7,:3p<Y.tYW*<",
    "UjI]\\6i&Pdz{yFIrrR6G7LDSm6",
    "ADY.qBN_k*N659S#idtLtQ5,}L8EaNtLTj<[`QX|dipnQfO*vJ=",
    "$3$2^,@9=a)R@](b!B8T%O;0#>O$4!Csar@4T2",
    "+atL$RBt!nY82Z9]",
    "T{L[1/?FE+s2Z@<cFw=}HppmD?!=M",
    "#1\"U*fy<`Rzi(Jxda&X0u&Kd-#.-YwS:1^QrS",
    "dXy@:@/cLYEHmE}yje]q|@''6vzbX/S,@sB31W&:",
    "x8$(A&zFFw#hQG;N",
    "-4[ne@|\"3N,?IhTyF^ZXE5[i[u[452f:S",
    "eFVy-#~`[,oIY\"j)GCn*uv\\`Z{SC*X>3.K\\pOc>-Mcs/R6GDsiAYa",
    "x}UY\"8d(x]EkS)Az;T'JOnPukUO`Tu8>}|W?)Z<18}7kQRx_o[rt[EpH!S",
    "L>8@l\\+Gw}d4>|adF(F9c<4\\,?U<~hfVlEV5",
    "MkB(Bs,P`u&gM9KuF!XImFg|HFkNIz:dv\\'GJj^Z+je=",
    "gLTU*CQ5r_+NXO&iLAsg9\\iNe_B?k72u_2rV_",
    "\"!Of}|B)0\"\\9XOypwGYa.iU~T+Am_-_L%{enH+mPSWbAie]c#r]AO[OHtvKUCc8",
    "Y1k/_fZV@V~UOx9lk#=",
    "N6|#1GD>tbj#vOA2:(@p",
    "ZfUX`taF$]/9jx$B{eMl.i;0x`v;qD:XK$;#5>Ut\\k~2]T9]#f$.Q[~",
    "J/O:sj==48Ce-#l+oB\\{&@yKw$m}isM",
    "usRPdfy3B-x8Dt~p;e,.f)1&1C,L#%M/",
    "}LatCGGQd3W/5d9#AB1=x^,|[>R.'plJeIQQmjNciAmt`J3gDC`qe",
    "~XK/}OuwVP-!L}^\\-icmoGhxZp-7I}0oWL5%TpBG-0^z",
    "j0&~o@4Y?zscWZEHcO(f)TsZSUdZxVY3F%5Ou@2|",
    "Z;b5DOd?qW|~\\+f?~",
    "E_:]d}o,ggJ;WsVKn]",
    "|#6XVR!WFJJBi9x+u\"H_uSI:)z",
    "uz;Z`&EDRR`[&HvIH@,$BH8",
    "\"L;$w+?3pBIJQm>$FC:R?L$vovffEEm*~*19/0{A+~iaI;0>t",
    "y@?jyRju1d\"LWNgS(\"b95D61[m>@6h6B$2q\"w\"8%v+]KTLbBaNse-B'/>.",
    "9Tn}IMHOa7n#\\Z]+Me)0-ov^@OcIFC;t<]5+n;",
    "[QPncuvPn6\"mD<$zAC4/rI%D7<T}q9pqx(Hz,F4:#;V!Obib`ta",
    "[0aXzQ8~EsKzS>u_*oZsq0gn8{Vq@N=v3,JqlBry",
    "B'#s7>G==B@,Xb&uAGQrz@uUFJBh?h_2fw\"jUT,`>odX=e:bC4fPCj",
    "-7|?ic7;Ogp-K:}HiZw6l;`(MbrMsc)lB{BVnH7_sc`);d+B~jzF",
    "t]7i}$N@2=C*&YDzf[+Tw=+\"j{/6%/=_k",
    "9:8;f305-U.LfHSAbS-IM)dDC'X2\\{L{!{m![2",
    "[oP~EE;@$wAJ-sGLt&q,w9=dHM):c&~799d=i`",
    "/@M=l86V]`N[cqbP42py",
    "}~+~@|y4xeY&4=?A\"",
];
