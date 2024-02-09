using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000084 RID: 132
	[HandlerCategory("vvIchimoku"), HandlerName("SilverSen")]
	public class Silversen : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000493 RID: 1171 RVA: 0x0001788C File Offset: 0x00015A8C
		public IList<double> Execute(ISecurity sec)
		{
			IList<double> highPrices = sec.get_HighPrices();
			IList<double> lowPrices = sec.get_LowPrices();
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> list = new List<double>(closePrices.Count);
			for (int i = 0; i < closePrices.Count; i++)
			{
				double item;
				if (i < this.SSPeriod)
				{
					item = 0.0;
				}
				else
				{
					double num = highPrices[i];
					double num2 = lowPrices[i];
					for (int j = i - this.SSPeriod + 1; j <= i; j++)
					{
						double num3 = highPrices[j];
						if (num < num3)
						{
							num = num3;
						}
						num3 = lowPrices[j];
						if (num2 > num3)
						{
							num2 = num3;
						}
					}
					item = num2 + (num - num2) * (double)this.SSK * 0.01;
				}
				list.Add(item);
			}
			return list;
		}

		// Token: 0x1700018E RID: 398
		public IContext Context
		{
			// Token: 0x06000494 RID: 1172 RVA: 0x00017967 File Offset: 0x00015B67
			get;
			// Token: 0x06000495 RID: 1173 RVA: 0x0001796F File Offset: 0x00015B6F
			set;
		}

		// Token: 0x1700018D RID: 397
		[HandlerParameter(true, "73.4", Min = "50", Max = "100", Step = "1")]
		public int SSK
		{
			// Token: 0x06000491 RID: 1169 RVA: 0x00017878 File Offset: 0x00015A78
			get;
			// Token: 0x06000492 RID: 1170 RVA: 0x00017880 File Offset: 0x00015A80
			set;
		}

		// Token: 0x1700018C RID: 396
		[HandlerParameter(true, "26", Min = "2", Max = "20", Step = "2")]
		public int SSPeriod
		{
			// Token: 0x0600048F RID: 1167 RVA: 0x00017867 File Offset: 0x00015A67
			get;
			// Token: 0x06000490 RID: 1168 RVA: 0x0001786F File Offset: 0x00015A6F
			set;
		}
	}
}
