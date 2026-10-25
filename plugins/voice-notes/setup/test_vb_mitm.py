import unittest

import vb_mitm

HEADER = "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode\n"
CLIENT = "   0: 0100007F:9C40 0100007F:22C1 01 00000000:00000000 00:00000000 00000000  1000        0 111 1 0 20 4 30 10 -1\n"
PROXY_SIDE = "   1: 0100007F:22C1 0100007F:9C40 01 00000000:00000000 00:00000000 00000000  1000        0 222 1 0 20 4 30 10 -1\n"
SAME_PORT_ELSEWHERE = "   2: 0A00000F:9C40 68104F1A:01BB 01 00000000:00000000 00:00000000 00000000  1000        0 333 1 0 20 4 30 10 -1\n"
LISTENER = "   3: 0100007F:22C1 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 444 1 0 100 0 0 10 0\n"


class ClientInodes(unittest.TestCase):
    def test_an_empty_table_has_no_client(self):
        self.assertEqual(vb_mitm.client_inodes(HEADER, 40000, 8897), set())

    def test_the_client_socket_is_the_one_connected_to_the_proxy(self):
        self.assertEqual(vb_mitm.client_inodes(HEADER + CLIENT + PROXY_SIDE + LISTENER, 40000, 8897), {"111"})

    def test_a_socket_elsewhere_with_the_same_local_port_is_not_the_client(self):
        self.assertEqual(vb_mitm.client_inodes(HEADER + SAME_PORT_ELSEWHERE + CLIENT, 40000, 8897), {"111"})


if __name__ == "__main__":
    unittest.main()
