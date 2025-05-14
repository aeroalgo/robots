using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000124 RID: 292
	[HandlerCategory("vvBands&Channels"), HandlerName("Price Channel Upper")]
	public class PCU : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000868 RID: 2152 RVA: 0x000236C0 File Offset: 0x000218C0
		public IList<double> Execute(IList<double> src)
		{
			double[] array = new double[src.Count];
			IList<double> data = this.Context.GetData("sma", new string[]
			{
				this.Period.ToString(),
				src.GetHashCode().ToString()
			}, () => Series.SMA(src, this.Period));
			for (int i = 0; i < src.Count; i++)
			{
				if (i < this.Period)
				{
					array[i] = 0.0;
				}
				else
				{
					array[i] = (1.0 + (double)(this.Sign * this.K) * 0.01) * data[i];
				}
			}
			return array;
		}

		// Token: 0x170002AE RID: 686
		public IContext Context
		{
			// Token: 0x06000869 RID: 2153 RVA: 0x0002379F File Offset: 0x0002199F
			get;
			// Token: 0x0600086A RID: 2154 RVA: 0x000237A7 File Offset: 0x000219A7
			set;
		}

		// Token: 0x170002AC RID: 684
		[HandlerParameter(true, "5", Min = "1", Max = "100", Step = "1")]
		public int K
		{
			// Token: 0x06000864 RID: 2148 RVA: 0x0002367C File Offset: 0x0002187C
			get;
			// Token: 0x06000865 RID: 2149 RVA: 0x00023684 File Offset: 0x00021884
			set;
		}

		// Token: 0x170002AD RID: 685
		[HandlerParameter(true, "20", Min = "5", Max = "100", Step = "5")]
		public int Period
		{
			// Token: 0x06000866 RID: 2150 RVA: 0x0002368D File Offset: 0x0002188D
			get;
			// Token: 0x06000867 RID: 2151 RVA: 0x00023695 File Offset: 0x00021895
			set;
		}

		// Token: 0x170002AB RID: 683
		[HandlerParameter(true, "1", Min = "-1", Max = "1", Step = "1")]
		public int Sign
		{
			// Token: 0x06000862 RID: 2146 RVA: 0x0002366B File Offset: 0x0002186B
			get;
			// Token: 0x06000863 RID: 2147 RVA: 0x00023673 File Offset: 0x00021873
			set;
		}
	}
}
