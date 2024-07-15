using System;
using System.Runtime.InteropServices;
using System.Text;

namespace CacClient
{
    public class Client {

        public void New(String tenant, CULong frequency, String hostName) {
            byte[] tenantBytes = Encoding.UTF8.GetBytes(tenant);
            byte[] hostNameBytes = Encoding.UTF8.GetBytes(hostName);
            cac_new_client(&tenantBytes[0], frequency, &hostNameBytes[0]);
        }
    }
}
