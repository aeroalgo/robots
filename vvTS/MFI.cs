using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000036 RID: 54
	[HandlerCategory("vvIndicators"), HandlerName("Money Flow Index")]
	public class MFI : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060001F3 RID: 499 RVA: 0x00009410 File Offset: 0x00007610
		public IList<double> Execute(ISecurity sec)
		{
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> highPrices = sec.get_HighPrices();
			IList<double> lowPrices = sec.get_LowPrices();
			IList<double> volumes = sec.get_Volumes();
			IList<double> list = new List<double>(closePrices.Count);
			IList<double> list2 = new List<double>(closePrices.Count);
			double num = 0.0;
			double num2 = 0.0;
			double item = volumes[0] * (highPrices[0] + closePrices[0] + lowPrices[0]) / 3.0;
			list2.Add(item);
			for (int i = 1; i < closePrices.Count; i++)
			{
				item = volumes[i] * (highPrices[i] + closePrices[i] + lowPrices[i]) / 3.0;
				list2.Add(item);
			}
			for (int j = 0; j < closePrices.Count; j++)
			{
				double item2;
				if (j < this.Period)
				{
					item2 = 0.0;
				}
				else
				{
					for (int k = j - this.Period + 1; k <= j; k++)
					{
						if (list2[j] >= list2[j - 1])
						{
							num += list2[k];
						}
						if (list2[j] < list2[j - 1])
						{
							num2 += list2[k];
						}
					}
					double num3;
					if (num2 != 0.0)
					{
						num3 = num / num2;
					}
					else
					{
						num3 = num / (num2 + 1E-08);
					}
					item2 = 100.0 - 100.0 / (1.0 + num3);
				}
				list.Add(item2);
			}
			return list;
		}

		// Token: 0x170000A9 RID: 169
		public IContext Context
		{
			// Token: 0x060001F4 RID: 500 RVA: 0x000095EC File Offset: 0x000077EC
			get;
			// Token: 0x060001F5 RID: 501 RVA: 0x000095F4 File Offset: 0x000077F4
			set;
		}

		// Token: 0x170000A8 RID: 168
		[HandlerParameter(true, "14", Min = "1", Max = "20", Step = "1")]
		public int Period
		{
			// Token: 0x060001F1 RID: 497 RVA: 0x000093FF File Offset: 0x000075FF
			get;
			// Token: 0x060001F2 RID: 498 RVA: 0x00009407 File Offset: 0x00007607
			set;
		}
	}
}
