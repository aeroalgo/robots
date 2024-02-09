using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000076 RID: 118
	[HandlerCategory("vvWilliams"), HandlerName("Williams A/D")]
	public class WAD : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600042E RID: 1070 RVA: 0x000165CC File Offset: 0x000147CC
		public IList<double> Execute(ISecurity sec)
		{
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> highPrices = sec.get_HighPrices();
			IList<double> lowPrices = sec.get_LowPrices();
			IList<double> list = new List<double>(closePrices.Count);
			double item = 0.0;
			list.Add(item);
			for (int i = 1; i < closePrices.Count; i++)
			{
				double num = Math.Max(highPrices[i], closePrices[i - 1]);
				double num2 = Math.Min(lowPrices[i], closePrices[i - 1]);
				double num3;
				if (closePrices[i] > closePrices[i - 1])
				{
					num3 = closePrices[i] - num2;
				}
				else if (closePrices[i] < closePrices[i - 1])
				{
					num3 = closePrices[i] - num;
				}
				else
				{
					num3 = 0.0;
				}
				item = list[i - 1] + num3;
				list.Add(item);
			}
			return JMA.GenJMA(list, this.Smooth, 100);
		}

		// Token: 0x1700016B RID: 363
		public IContext Context
		{
			// Token: 0x0600042F RID: 1071 RVA: 0x000166D0 File Offset: 0x000148D0
			get;
			// Token: 0x06000430 RID: 1072 RVA: 0x000166D8 File Offset: 0x000148D8
			set;
		}

		// Token: 0x1700016A RID: 362
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "1")]
		public int Smooth
		{
			// Token: 0x0600042C RID: 1068 RVA: 0x000165B8 File Offset: 0x000147B8
			get;
			// Token: 0x0600042D RID: 1069 RVA: 0x000165C0 File Offset: 0x000147C0
			set;
		}
	}
}
