using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000132 RID: 306
	[HandlerCategory("vvRSI"), HandlerName("RapidRSI")]
	public class RapidRSI : BasePeriodIndicatorHandler, IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600090A RID: 2314 RVA: 0x00026205 File Offset: 0x00024405
		public IList<double> Execute(IList<double> src)
		{
			return this.GenRapidRSI(src, base.get_Period());
		}

		// Token: 0x06000909 RID: 2313 RVA: 0x00026144 File Offset: 0x00024344
		public IList<double> GenRapidRSI(IList<double> src, int period)
		{
			double[] array = new double[src.Count];
			for (int i = period; i < src.Count; i++)
			{
				double num = 0.0;
				double num2 = 0.0;
				for (int j = i - period + 1; j <= i; j++)
				{
					double num3 = src[j] - src[j - 1];
					if (num3 > 0.0)
					{
						num += num3;
					}
					else
					{
						num2 += -num3;
					}
				}
				if (num + num2 == 0.0)
				{
					array[i] = 50.0;
				}
				else
				{
					array[i] = 100.0 * num / (num + num2);
				}
			}
			return array;
		}

		// Token: 0x170002E8 RID: 744
		public IContext Context
		{
			// Token: 0x0600090B RID: 2315 RVA: 0x00026214 File Offset: 0x00024414
			get;
			// Token: 0x0600090C RID: 2316 RVA: 0x0002621C File Offset: 0x0002441C
			set;
		}
	}
}
