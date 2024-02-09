using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000156 RID: 342
	[HandlerCategory("vvAverages"), HandlerName("AMMA")]
	public class AMMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000ABE RID: 2750 RVA: 0x0002C704 File Offset: 0x0002A904
		public IList<double> Execute(IList<double> price)
		{
			return this.Context.GetData("AMMA", new string[]
			{
				this.Period.ToString(),
				price.GetHashCode().ToString()
			}, () => AMMA.GenAMMA(price, this.Period));
		}

		// Token: 0x06000ABD RID: 2749 RVA: 0x0002C690 File Offset: 0x0002A890
		public static IList<double> GenAMMA(IList<double> src, int period)
		{
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				if (i < 2)
				{
					array[i] = src[i];
				}
				else
				{
					array[i] = ((double)(period - 1) * array[i - 1] + src[i]) / (double)period;
				}
			}
			return array;
		}

		// Token: 0x1700038F RID: 911
		public IContext Context
		{
			// Token: 0x06000ABF RID: 2751 RVA: 0x0002C770 File Offset: 0x0002A970
			get;
			// Token: 0x06000AC0 RID: 2752 RVA: 0x0002C778 File Offset: 0x0002A978
			set;
		}

		// Token: 0x1700038E RID: 910
		[HandlerParameter(true, "25", Min = "10", Max = "50", Step = "1")]
		public int Period
		{
			// Token: 0x06000ABB RID: 2747 RVA: 0x0002C67C File Offset: 0x0002A87C
			get;
			// Token: 0x06000ABC RID: 2748 RVA: 0x0002C684 File Offset: 0x0002A884
			set;
		}
	}
}
