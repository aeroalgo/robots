using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000024 RID: 36
	[HandlerCategory("vvIndicators"), HandlerName("Ergodic")]
	public class Ergodic : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000150 RID: 336 RVA: 0x0000649B File Offset: 0x0000469B
		public IList<double> Execute(IList<double> src)
		{
			return Ergodic.GenErgodicFXS(src, this.Context, this.pq, this.pr, this.ps, this.trigger, this.output, this.Chart);
		}

		// Token: 0x0600014F RID: 335 RVA: 0x00006118 File Offset: 0x00004318
		public static IList<double> GenErgodicFXS(IList<double> src, IContext ctx, int _pq, int _pr, int _ps, int _trigger, int _output, int _chart)
		{
			int count = src.Count;
			double[,] array = new double[count, 8];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double num = 2.0 / (1.0 + (double)_pq);
			double num2 = 2.0 / (1.0 + (double)_pr);
			double num3 = 2.0 / (1.0 + (double)_ps);
			double num4 = 2.0 / (1.0 + (double)_trigger);
			array[0, 0] = src[0];
			for (int i = 1; i < count; i++)
			{
				array[i, 0] = src[i] - src[i - 1];
				array[i, 1] = Math.Abs(array[i, 0]);
				array[i, 2] = array[i - 1, 2] + num * (array[i, 0] - array[i - 1, 2]);
				array[i, 3] = array[i - 1, 3] + num2 * (array[i, 2] - array[i - 1, 3]);
				array[i, 4] = array[i - 1, 4] + num * (array[i, 1] - array[i - 1, 4]);
				array[i, 5] = array[i - 1, 5] + num2 * (array[i, 4] - array[i - 1, 5]);
				array[i, 6] = array[i - 1, 6] + num3 * (array[i, 3] - array[i - 1, 6]);
				array[i, 7] = array[i - 1, 7] + num3 * (array[i, 5] - array[i - 1, 7]);
				if (array[i, 7] != 0.0)
				{
					array2[i] = 500.0 * array[i, 6] / array[i, 7];
				}
				else
				{
					array2[i] = 0.0;
				}
				array3[i] = array3[i - 1] + num4 * (array2[i] - array3[i - 1]);
				array4[i] = 0.0;
				if (array2[i] > array3[i] && array2[i - 1] < array3[i - 1])
				{
					array4[i] = 1.0;
				}
				if (array2[i] < array3[i] && array2[i - 1] > array3[i - 1])
				{
					array4[i] = -1.0;
				}
			}
			if (_chart > 0)
			{
				IPane pane = ctx.CreatePane("", 40.0, false, false);
				pane.AddList(string.Concat(new string[]
				{
					"Ergo(pq:",
					_pq.ToString(),
					",pr:",
					_pr.ToString(),
					",ps:",
					_ps.ToString(),
					")"
				}), array2, 0, 331645, 0, 0);
				pane.AddList("Trigger(" + _trigger.ToString() + ")", array3, 0, 14026756, 0, 0);
			}
			if (_output == 1)
			{
				return array3;
			}
			if (_output == 2)
			{
				return array4;
			}
			return array2;
		}

		// Token: 0x1700006F RID: 111
		[HandlerParameter(true, "0", NotOptimized = true)]
		public int Chart
		{
			// Token: 0x0600014D RID: 333 RVA: 0x00006106 File Offset: 0x00004306
			get;
			// Token: 0x0600014E RID: 334 RVA: 0x0000610E File Offset: 0x0000430E
			set;
		}

		// Token: 0x17000070 RID: 112
		public IContext Context
		{
			// Token: 0x06000151 RID: 337 RVA: 0x000064CD File Offset: 0x000046CD
			get;
			// Token: 0x06000152 RID: 338 RVA: 0x000064D5 File Offset: 0x000046D5
			set;
		}

		// Token: 0x1700006E RID: 110
		[HandlerParameter(true, "0", NotOptimized = true)]
		public int output
		{
			// Token: 0x0600014B RID: 331 RVA: 0x000060F5 File Offset: 0x000042F5
			get;
			// Token: 0x0600014C RID: 332 RVA: 0x000060FD File Offset: 0x000042FD
			set;
		}

		// Token: 0x1700006A RID: 106
		[HandlerParameter(true, "2", Min = "1", Max = "10", Step = "1")]
		public int pq
		{
			// Token: 0x06000143 RID: 323 RVA: 0x000060B1 File Offset: 0x000042B1
			get;
			// Token: 0x06000144 RID: 324 RVA: 0x000060B9 File Offset: 0x000042B9
			set;
		}

		// Token: 0x1700006B RID: 107
		[HandlerParameter(true, "10", Min = "1", Max = "10", Step = "1")]
		public int pr
		{
			// Token: 0x06000145 RID: 325 RVA: 0x000060C2 File Offset: 0x000042C2
			get;
			// Token: 0x06000146 RID: 326 RVA: 0x000060CA File Offset: 0x000042CA
			set;
		}

		// Token: 0x1700006C RID: 108
		[HandlerParameter(true, "5", Min = "1", Max = "10", Step = "1")]
		public int ps
		{
			// Token: 0x06000147 RID: 327 RVA: 0x000060D3 File Offset: 0x000042D3
			get;
			// Token: 0x06000148 RID: 328 RVA: 0x000060DB File Offset: 0x000042DB
			set;
		}

		// Token: 0x1700006D RID: 109
		[HandlerParameter(true, "3", Min = "1", Max = "10", Step = "1")]
		public int trigger
		{
			// Token: 0x06000149 RID: 329 RVA: 0x000060E4 File Offset: 0x000042E4
			get;
			// Token: 0x0600014A RID: 330 RVA: 0x000060EC File Offset: 0x000042EC
			set;
		}
	}
}
