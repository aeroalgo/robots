using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200003D RID: 61
	[HandlerCategory("vvIndicators"), HandlerName("On Balance Volume")]
	public class OBV : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000231 RID: 561 RVA: 0x0000A30C File Offset: 0x0000850C
		public IList<double> Execute(ISecurity sec)
		{
			IList<double> volumes = sec.get_Volumes();
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> list = new List<double>(closePrices.Count);
			double item = 0.0;
			list.Add(volumes[0]);
			for (int i = 1; i < closePrices.Count; i++)
			{
				if (closePrices[i] > closePrices[i - 1])
				{
					item = list[i - 1] + volumes[i];
				}
				if (closePrices[i] < closePrices[i - 1])
				{
					item = list[i - 1] - volumes[i];
				}
				if (closePrices[i] == closePrices[i - 1])
				{
					item = list[i - 1];
				}
				list.Add(item);
			}
			return list;
		}

		// Token: 0x170000BE RID: 190
		public IContext Context
		{
			// Token: 0x06000232 RID: 562 RVA: 0x0000A3D9 File Offset: 0x000085D9
			get;
			// Token: 0x06000233 RID: 563 RVA: 0x0000A3E1 File Offset: 0x000085E1
			set;
		}
	}
}
